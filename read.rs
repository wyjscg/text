use kube::api::Api;
use kube::Client;
use k8s_openapi::api::core::v1::Secret;
use chrono::{DateTime, Utc};

pub struct AuthenticatorResponse {
    pub user: UserInfo,
}

pub struct UserInfo {
    pub name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    
    #[error("Token parse error: {0}")]
    TokenParseError(String),
}

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

pub mod bootstrap_token_util {
    use super::AuthError;

    pub fn parse_token(token: &str) -> Result<(String, String), AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        
        if parts.len() != 2 {
            return Err(AuthError::TokenParseError(
                "token must be of form '<token-id>.<token-secret>'".to_string()
            ));
        }

        let token_id = parts[0];
        let token_secret = parts[1];

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

pub mod bootstrap_secret_util {
    use super::*;
    use super::bootstrap_api::*;

    pub fn get_data(secret: &Secret, key: &str) -> String {
        secret.data.as_ref()
            .and_then(|data| data.get(key))
            .and_then(|bytes| String::from_utf8(bytes.0.clone()).ok())
            .unwrap_or_default()
    }

    pub fn has_expired(secret: &Secret, now: DateTime<Utc>) -> bool {
        let expiration = get_data(secret, BOOTSTRAP_TOKEN_EXPIRATION_KEY);
        
        if expiration.is_empty() {
            return false;
        }

        match DateTime::parse_from_rfc3339(&expiration) {
            Ok(exp_time) => exp_time.with_timezone(&Utc) < now,
            Err(_) => true,
        }
    }

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

        groups.push("system:bootstrappers".to_string());
        groups.push("system:authenticated".to_string());

        Ok(groups)
    }
}

fn token_errorf(secret: &Secret, message: &str) {
    let secret_name = secret.metadata.name.as_deref().unwrap_or("unknown");
    log::warn!("Bootstrap token secret {} {}", secret_name, message);
}

// TODO: 此包中的一些方法是从其他来源复制的。
// 要么是因为现有功能未导出，要么是因为它位于不应该被此包直接导入的包中。

// 初始化一个 bootstrap token 认证器。
//
// Lister 预期作用于 "kube-system" 命名空间。
pub fn new_token_authenticator(client: Client) -> TokenAuthenticator {
    TokenAuthenticator { 
        lister: Api::namespaced(client, "kube-system") 
    }
}

// TokenAuthenticator 从 API 服务器中的 secrets 认证 bootstrap tokens。
pub struct TokenAuthenticator {
    lister: Api<Secret>,
}

impl TokenAuthenticator {
    pub async fn authenticate_token(
        &self,
        token: &str,
    ) -> Result<(Option<AuthenticatorResponse>, bool), AuthError> {
        let (token_id, token_secret) = match bootstrap_token_util::parse_token(token) {
            Ok(parts) => parts,
            Err(_) => {
                // Token 格式不正确，忽略它。
                return Ok((None, false));
            }
        };

        let secret_name = format!("{}{}", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_PREFIX, token_id);
        let secret = match self.lister.get(&secret_name).await {
            Ok(s) => s,
            Err(e) => {
                if let kube::Error::Api(ref response) = e {
                    if response.code == 404 {
                        log::info!("No secret of name {} to match bootstrap bearer token", secret_name);
                        return Ok((None, false));
                    }
                }
                return Err(AuthError::KubeError(e));
            }
        };

        if secret.metadata.deletion_timestamp.is_some() {
            token_errorf(&secret, "is deleted and awaiting removal");
            return Ok((None, false));
        }

        if secret.type_.as_deref() != Some(bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN) 
            || secret.data.is_none() {
            token_errorf(
                &secret,
                &format!("has invalid type, expected {}.", bootstrap_api::SECRET_TYPE_BOOTSTRAP_TOKEN)
            );
            return Ok((None, false));
        }

        let ts = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY);
        if !constant_time_compare(&ts, &token_secret) {
            token_errorf(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_SECRET_KEY)
            );
            return Ok((None, false));
        }

        let id = bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY);
        if id != token_id {
            token_errorf(
                &secret,
                &format!("has invalid value for key {}.", bootstrap_api::BOOTSTRAP_TOKEN_ID_KEY)
            );
            return Ok((None, false));
        }

        if bootstrap_secret_util::has_expired(&secret, Utc::now()) {
            // 日志记录在 isSecretExpired 方法中完成。
            return Ok((None, false));
        }

        if bootstrap_secret_util::get_data(&secret, bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION) != "true" {
            token_errorf(
                &secret,
                &format!("not marked {}=true.", bootstrap_api::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION)
            );
            return Ok((None, false));
        }

        let groups = match bootstrap_secret_util::get_groups(&secret) {
            Ok(g) => g,
            Err(e) => {
                token_errorf(
                    &secret,
                    &format!("has invalid value for key {}: {}.", 
                        bootstrap_api::BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY, e)
                );
                return Ok((None, false));
            }
        };

        Ok((
            Some(AuthenticatorResponse {
                user: UserInfo {
                    name: format!("{}{}", bootstrap_api::BOOTSTRAP_USER_PREFIX, id),
                    groups,
                },
            }),
            true,
        ))
    }
}

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
