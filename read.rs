use kube::api::Api;
use kube::Client;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 认证响应结构（对应 Go 的 authenticator.Response）
#[derive(Debug, Clone)]
pub struct AuthenticatorResponse {
    pub user: UserInfo,
}

/// 用户信息（对应 Go 的 user.DefaultInfo）
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub name: String,
    pub groups: Vec<String>,
}

/// 认证错误类型
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    
    #[error("Token parse error: {0}")]
    TokenParseError(String),
}

/// Bootstrap token 相关常量
pub mod bootstrap_api {
    pub const BOOTSTRAP_TOKEN_SECRET_PREFIX: &str = "bootstrap-token-";
    pub const SECRET_TYPE_BOOTSTRAP_TOKEN: &str = "bootstrap.kubernetes.io/token";
    pub const BOOTSTRAP_TOKEN_SECRET_KEY: &str = "token-secret";
    pub const BOOTSTRAP_TOKEN_ID_KEY: &str = "token-id";
    pub const BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION: &str = "usage-bootstrap-authentication";
    pub const BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY: &str = "auth-extra-groups";
    pub const BOOTSTRAP_TOKEN_EXPIRATION_KEY: &str = "expiration";
    pub const BOOTSTRAP_USER_PREFIX: &str = "system:bootstrap:";
}

/// Bootstrap token 工具函数模块
pub mod bootstrap_token_util {
    use super::AuthError;

    /// 解析 token
    /// 对应 Go: bootstraptokenutil.ParseToken(token)
    /// 返回: Result<(tokenID, tokenSecret), error>
    pub fn parse_token(token: &str) -> Result<(String, String), AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        
        if parts.len() != 2 {
            return Err(AuthError::TokenParseError(
                "token must be of form '<token-id>.<token-secret>'".to_string()
            ));
        }

        let token_id = parts[0];
        let token_secret = parts[1];

        // 验证格式
        if token_id.len() != 6 || !token_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(AuthError::TokenParseError(
                "invalid token ID format".to_string()
            ));
        }

        if token_secret.len() != 16 || !token_secret.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(AuthError::TokenParseError(
                "invalid token secret format".to_string()
            ));
        }

        Ok((token_id.to_string(), token_secret.to_string()))
    }
}

/// Bootstrap secret 工具函数模块
pub mod bootstrap_secret_util {
    use super::*;
    use super::bootstrap_api::*;

    /// 从 secret 中获取数据
    /// 对应 Go: bootstrapsecretutil.GetData(secret, key)
    pub fn get_data(secret: &Secret, key: &str) -> String {
        secret.data.as_ref()
            .and_then(|data| data.get(key))
            .and_then(|bytes| String::from_utf8(bytes.0.clone()).ok())
            .unwrap_or_default()
    }

    /// 检查 secret 是否已过期
    /// 对应 Go: bootstrapsecretutil.HasExpired(secret, time.Now())
    pub fn has_expired(secret: &Secret, now: DateTime<Utc>) -> bool {
        let expiration = get_data(secret, BOOTSTRAP_TOKEN_EXPIRATION_KEY);
        
        if expiration.is_empty() {
            return false;
        }

        match DateTime::parse_from_rfc3339(&expiration) {
            Ok(exp_time) => {
                let expired = exp_time.with_timezone(&Utc) < now;
                if expired {
                    // 对应 Go: logging done in isSecretExpired method
                    log::info!("Bootstrap token in secret {} has expired", 
                        secret.metadata.name.as_deref().unwrap_or("unknown"));
                }
                expired
            }
            Err(_) => true,
        }
    }

    /// 获取用户组列表
    /// 对应 Go: bootstrapsecretutil.GetGroups(secret)
    pub fn get_groups(secret: &Secret) -> Result<Vec<String>, AuthError> {
        let groups_str = get_data(secret, BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY);
        
        let mut groups: Vec<String> = if groups_str.is_empty() {
            Vec::new()
        } else {
            groups_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        // 添加默认组
        groups.push("system:bootstrappers".to_string());
        groups.push("system:authenticated".to_string());

        Ok(groups)
    }
}

/// TokenAuthenticator 从 API 服务器中的 secrets 认证 bootstrap tokens
pub struct TokenAuthenticator {
    /// 用于查询 kube-system 命名空间中 Secret 的 API 客户端
    /// 对应 Go: lister corev1listers.SecretNamespaceLister
    secret_api: Api<Secret>,
}

impl TokenAuthenticator {
    /// 创建一个新的 TokenAuthenticator 实例
    /// 对应 Go: NewTokenAuthenticator(lister corev1listers.SecretNamespaceLister)
    pub fn new(client: Client) -> Self {
        Self {
            secret_api: Api::namespaced(client, "kube-system"),
        }
    }

    /// 认证 token
    /// 
    /// 对应 Go 函数签名:
    /// func (t *TokenAuthenticator) AuthenticateToken(ctx context.Context, token string) 
    ///     (*authenticator.Response, bool, error)
    /// 
    /// 返回值语义:
    /// - Ok((Some(response), true)) - 认证成功，返回响应
    /// - Ok((None, false)) - 认证失败（token 无效），不是错误
    /// - Err(error) - 发生真正的错误
    pub async fn authenticate_token(
        &self,
        token: &str,
    ) -> Result<(Option<AuthenticatorResponse>, bool), AuthError> {
        // 对应 Go: tokenID, tokenSecret, err := bootstraptokenutil.ParseToken(token)
        let (token_id, token_secret) = match bootstrap_token_util::parse_token(token) {
            Ok(parts) => parts,
            Err(_) => {
                // 对应 Go: if err != nil { return nil, false, nil }
                // Token isn't of the correct form, ignore it.
                return Ok((None, false));
            }
        };

        // 对应 Go: secretName := bootstrapapi.BootstrapTokenSecretPrefix + tokenID
        let secret_name = format!("{}{}", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_PREFIX, token_id);

        // 对应 Go: secret, err := t.lister.Get(secretName)
        let secret = match self.secret_api.get(&secret_name).await {
            Ok(s) => s,
            Err(e) => {
                // 对应 Go: if errors.IsNotFound(err)
                if let kube::Error::Api(ref response) = e {
                    if response.code == 404 {
                        // 对应 Go: klog.V(3).Infof(...)
                        log::info!("No secret of name {} to match bootstrap bearer token", secret_name);
                        // 对应 Go: return nil, false, nil
                        return Ok((None, false));
                    }
                }
                // 对应 Go: return nil, false, err
                return Err(AuthError::KubeError(e));
            }
        };

        // 对应 Go: if secret.DeletionTimestamp != nil
        if secret.metadata.deletion_timestamp.is_some() {
            // 对应 Go: tokenErrorf(secret, "is deleted and awaiting removal")
            Self::token_errorf(&secret, "is deleted and awaiting removal");
            // 对应 Go: return nil, false, nil
            return Ok((None, false));
        }

        // 对应 Go: if string(secret.Type) != string(bootstrapapi.SecretTypeBootstrapToken) || secret.Data == nil
        if secret.type_.as_deref() != Some(bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN) 
            || secret.data.is_none() {
            // 对应 Go: tokenErrorf(secret, "has invalid type, expected %s.", ...)
            Self::token_errorf(
                &secret,
                &format!("has invalid type, expected {}.", bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN)
            );
            // 对应 Go: return nil, false, nil
            return Ok((None, false));
        }

        // 对应 Go: ts := bootstrapsecretutil.GetData(secret, bootstrapapi.BootstrapTokenSecretKey)
        let ts = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY);
        
        // 对应 Go: if subtle.ConstantTimeCompare([]byte(ts), []byte(tokenSecret)) != 1
        if !Self::constant_time_compare(&ts, &token_secret) {
            // 对应 Go: tokenErrorf(secret, "has invalid value for key %s.", ...)
            Self::token_errorf(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY)
            );
            // 对应 Go: return nil, false, nil
            return Ok((None, false));
        }

        // 对应 Go: id := bootstrapsecretutil.GetData(secret, bootstrapapi.BootstrapTokenIDKey)
        let id = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY);
        
        // 对应 Go: if id != tokenID
        if id != token_id {
            // 对应 Go: tokenErrorf(secret, "has invalid value for key %s.", ...)
            Self::token_errorf(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY)
            );
            // 对应 Go: return nil, false, nil
            return Ok((None, false));
        }

        // 对应 Go: if bootstrapsecretutil.HasExpired(secret, time.Now())
        if bootstrap_secret_util::has_expired(&secret, Utc::now()) {
            // 对应 Go: return nil, false, nil
            // logging done in isSecretExpired method.
            return Ok((None, false));
        }

        // 对应 Go: if bootstrapsecretutil.GetData(secret, bootstrapapi.BootstrapTokenUsageAuthentication) != "true"
        if bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION) != "true" {
            // 对应 Go: tokenErrorf(secret, "not marked %s=true.", ...)
            Self::token_errorf(
                &secret,
                &format!("not marked {}=true.", bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION)
            );
            // 对应 Go: return nil, false, nil
            return Ok((None, false));
        }

        // 对应 Go: groups, err := bootstrapsecretutil.GetGroups(secret)
        let groups = match bootstrap_secret_util::get_groups(&secret) {
            Ok(g) => g,
            Err(e) => {
                // 对应 Go: tokenErrorf(secret, "has invalid value for key %s: %v.", ...)
                Self::token_errorf(
                    &secret,
                    &format!("has invalid value for key {}: {}.", 
                        bootstrap_api::BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY, e)
                );
                // 对应 Go: return nil, false, nil
                return Ok((None, false));
            }
        };

        // 对应 Go: return &authenticator.Response{User: &user.DefaultInfo{...}}, true, nil
        Ok((
            Some(AuthenticatorResponse {
                user: UserInfo {
                    // 对应 Go: Name: bootstrapapi.BootstrapUserPrefix + string(id)
                    name: format!("{}{}", bootstrap_api::BOOTSTRAP_USER_PREFIX, id),
                    // 对应 Go: Groups: groups
                    groups,
                },
            }),
            true,
        ))
    }

    /// 恒定时间字符串比较（防止时序攻击）
    /// 对应 Go: subtle.ConstantTimeCompare([]byte(a), []byte(b))
    fn constant_time_compare(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        
        let mut result = 0u8;
        for i in 0..a_bytes.len() {
            result |= a_bytes[i] ^ b_bytes[i];
        }
        
        result == 0
    }

    /// 记录 token 错误
    /// 对应 Go: tokenErrorf(secret, format, args...)
    fn token_errorf(secret: &Secret, message: &str) {
        let secret_name = secret.metadata.name.as_deref().unwrap_or("unknown");
        log::warn!("Bootstrap token secret {} {}", secret_name, message);
    }
}

// ============================================================================
// 使用示例
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate_token_usage() {
        let client = Client::try_default().await.expect("创建客户端失败");
        let authenticator = TokenAuthenticator::new(client);

        // 对应 Go 的调用方式:
        // response, authenticated, err := authenticator.AuthenticateToken(ctx, token)
        match authenticator.authenticate_token("abcdef.0123456789abcdef").await {
            // 对应 Go: if err != nil
            Err(e) => {
                eprintln!("错误: {}", e);
            }
            // 对应 Go: if authenticated && response != nil
            Ok((Some(response), true)) => {
                println!("认证成功！");
                println!("用户名: {}", response.user.name);
                println!("用户组: {:?}", response.user.groups);
            }
            // 对应 Go: if !authenticated
            Ok((None, false)) => {
                println!("认证失败：token 无效");
            }
            // 这种情况理论上不应该出现（与 Go 代码逻辑一致）
            Ok((Some(_), false)) | Ok((None, true)) => {
                println!("不一致的状态（不应该发生）");
            }
        }
    }

    #[test]
    fn test_parse_token() {
        // 测试有效 token
        let result = bootstrap_token_util::parse_token("abcdef.0123456789abcdef");
        assert!(result.is_ok());
        let (id, secret) = result.unwrap();
        assert_eq!(id, "abcdef");
        assert_eq!(secret, "0123456789abcdef");

        // 测试无效 token
        assert!(bootstrap_token_util::parse_token("invalid").is_err());
        assert!(bootstrap_token_util::parse_token("abc.def").is_err());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(TokenAuthenticator::constant_time_compare("hello", "hello"));
        assert!(!TokenAuthenticator::constant_time_compare("hello", "world"));
        assert!(!TokenAuthenticator::constant_time_compare("hello", "hello!"));
    }
}
