use kube::api::Api;
use kube::Client;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};

/// 认证响应结构
#[derive(Debug, Clone)]
pub struct AuthenticatorResponse {
    pub user: UserInfo,
}

/// 用户信息
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
    
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
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

/// Bootstrap token 工具函数
pub mod bootstrap_token_util {
    use super::AuthError;

    /// 解析 token，返回 (token_id, token_secret)
    pub fn parse_token(token: &str) -> Result<(String, String), AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        
        if parts.len() != 2 {
            return Err(AuthError::TokenParseError(
                "token must be of form '<token-id>.<token-secret>'".to_string()
            ));
        }

        let token_id = parts[0];
        let token_secret = parts[1];

        // 验证 token ID 格式 (6 个字符，小写字母数字)
        if token_id.len() != 6 || !token_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(AuthError::TokenParseError(
                "token ID must be 6 lowercase alphanumeric characters".to_string()
            ));
        }

        // 验证 token secret 格式 (16 个字符，小写字母数字)
        if token_secret.len() != 16 || !token_secret.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(AuthError::TokenParseError(
                "token secret must be 16 lowercase alphanumeric characters".to_string()
            ));
        }

        Ok((token_id.to_string(), token_secret.to_string()))
    }
}

/// Bootstrap secret 工具函数
pub mod bootstrap_secret_util {
    use super::*;
    use super::bootstrap_api::*;

    /// 从 secret 中获取数据
    pub fn get_data(secret: &Secret, key: &str) -> String {
        secret.data.as_ref()
            .and_then(|data| data.get(key))
            .and_then(|bytes| String::from_utf8(bytes.0.clone()).ok())
            .unwrap_or_default()
    }

    /// 检查 secret 是否已过期
    pub fn has_expired(secret: &Secret, now: DateTime<Utc>) -> bool {
        let expiration = get_data(secret, BOOTSTRAP_TOKEN_EXPIRATION_KEY);
        
        if expiration.is_empty() {
            return false; // 没有过期时间表示永不过期
        }

        // 解析 RFC3339 格式的时间
        match DateTime::parse_from_rfc3339(&expiration) {
            Ok(exp_time) => {
                let expired = exp_time.with_timezone(&Utc) < now;
                if expired {
                    log::info!("Bootstrap token in secret {} has expired", 
                        secret.metadata.name.as_deref().unwrap_or("unknown"));
                }
                expired
            }
            Err(e) => {
                log::warn!("Invalid expiration time format in secret {}: {}", 
                    secret.metadata.name.as_deref().unwrap_or("unknown"), e);
                true // 无法解析则视为已过期
            }
        }
    }

    /// 获取用户组列表
    pub fn get_groups(secret: &Secret) -> Result<Vec<String>, AuthError> {
        let groups_str = get_data(secret, BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY);
        
        if groups_str.is_empty() {
            // 默认组
            return Ok(vec![
                "system:bootstrappers".to_string(),
                "system:authenticated".to_string(),
            ]);
        }

        // 解析逗号分隔的组列表
        let mut groups: Vec<String> = groups_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 添加默认组
        groups.push("system:bootstrappers".to_string());
        groups.push("system:authenticated".to_string());

        Ok(groups)
    }
}

impl TokenAuthenticator {
    /// 认证 token
    ///
    /// # 返回
    /// - Ok(Some(response)) - 认证成功
    /// - Ok(None) - 认证失败（token 无效）
    /// - Err(error) - 发生错误
    pub async fn authenticate_token(
        &self,
        token: &str,
    ) -> Result<Option<AuthenticatorResponse>, AuthError> {
        // 解析 token
        let (token_id, token_secret) = match bootstrap_token_util::parse_token(token) {
            Ok(parts) => parts,
            Err(_) => {
                // Token 格式不正确，忽略它
                return Ok(None);
            }
        };

        // 构造 secret 名称
        let secret_name = format!("{}{}", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_PREFIX, token_id);

        // 获取 secret
        let secret = match self.secret_api.get(&secret_name).await {
            Ok(s) => s,
            Err(kube::Error::Api(response)) if response.code == 404 => {
                log::info!("No secret of name {} to match bootstrap bearer token", secret_name);
                return Ok(None);
            }
            Err(e) => return Err(AuthError::KubeError(e)),
        };

        // 检查 secret 是否正在被删除
        if secret.metadata.deletion_timestamp.is_some() {
            Self::token_error(&secret, "is deleted and awaiting removal");
            return Ok(None);
        }

        // 检查 secret 类型
        if secret.type_.as_deref() != Some(bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN) 
            || secret.data.is_none() {
            Self::token_error(
                &secret,
                &format!("has invalid type, expected {}.", bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN)
            );
            return Ok(None);
        }

        // 验证 token secret（使用恒定时间比较防止时序攻击）
        let ts = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY);
        if !Self::constant_time_compare(&ts, &token_secret) {
            Self::token_error(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY)
            );
            return Ok(None);
        }

        // 验证 token ID
        let id = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY);
        if id != token_id {
            Self::token_error(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY)
            );
            return Ok(None);
        }

        // 检查是否过期
        if bootstrap_secret_util::has_expired(&secret, Utc::now()) {
            return Ok(None);
        }

        // 检查是否标记为可用于认证
        let usage = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION);
        if usage != "true" {
            Self::token_error(
                &secret,
                &format!("not marked {}=true.", bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION)
            );
            return Ok(None);
        }

        // 获取用户组
        let groups = match bootstrap_secret_util::get_groups(&secret) {
            Ok(g) => g,
            Err(e) => {
                Self::token_error(
                    &secret,
                    &format!("has invalid value for key {}: {}.", 
                        bootstrap_api::BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY, e)
                );
                return Ok(None);
            }
        };

        // 构造认证响应
        Ok(Some(AuthenticatorResponse {
            user: UserInfo {
                name: format!("{}{}", bootstrap_api::BOOTSTRAP_USER_PREFIX, id),
                groups,
            },
        }))
    }

    /// 恒定时间字符串比较（防止时序攻击）
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
    fn token_error(secret: &Secret, message: &str) {
        let secret_name = secret.metadata.name.as_deref().unwrap_or("unknown");
        log::warn!("Bootstrap token secret {} {}", secret_name, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token() {
        // 有效的 token
        let result = bootstrap_token_util::parse_token("abcdef.0123456789abcdef");
        assert!(result.is_ok());
        let (id, secret) = result.unwrap();
        assert_eq!(id, "abcdef");
        assert_eq!(secret, "0123456789abcdef");

        // 无效的 token
        assert!(bootstrap_token_util::parse_token("invalid").is_err());
        assert!(bootstrap_token_util::parse_token("abc.def").is_err());
        assert!(bootstrap_token_util::parse_token("ABCDEF.0123456789abcdef").is_err());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(TokenAuthenticator::constant_time_compare("hello", "hello"));
        assert!(!TokenAuthenticator::constant_time_compare("hello", "world"));
        assert!(!TokenAuthenticator::constant_time_compare("hello", "hello!"));
    }

    #[tokio::test]
    async fn test_authenticate_token() {
        let client = Client::try_default().await.expect("创建客户端失败");
        let authenticator = TokenAuthenticator::new(client);

        let result = authenticator
            .authenticate_token("abcdef.0123456789abcdef")
            .await;

        match result {
            Ok(Some(response)) => {
                println!("认证成功！用户: {}", response.user.name);
                println!("用户组: {:?}", response.user.groups);
            }
            Ok(None) => println!("认证失败：token 无效"),
            Err(e) => eprintln!("错误: {}", e),
        }
    }
}
