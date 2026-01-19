fn authenticate_token(
    &self,
    ctx: context::Context,
    token: String,
) -> Result<(Option<authenticator::Response>, bool), Box<dyn std::error::Error>> {
    let (token_id, token_secret) = match bootstraptokenutil::parse_token(&token) {
        Ok((id, secret)) => (id, secret),
        Err(_err) => {
            // Token 格式不正确，忽略它。
            return Ok((None, false));
        }
    };

    let secret_name = format!("{}{}", bootstrapapi::BOOTSTRAP_TOKEN_SECRET_PREFIX, token_id);
    let secret = match self.lister.get(&secret_name) {
        Ok(s) => s,
        Err(err) => {
            if errors::is_not_found(&err) {
                klog::v(3).infof("No secret of name %s to match bootstrap bearer token", &secret_name);
                return Ok((None, false));
            }
            return Err(err);
        }
    };

    if secret.deletion_timestamp.is_some() {
        token_errorf(&secret, "is deleted and awaiting removal");
        return Ok((None, false));
    }

    if secret.r#type.as_deref() != Some(bootstrapapi::SECRET_TYPE_BOOTSTRAP_TOKEN) 
        || secret.data.is_none() {
        token_errorf(&secret, &format!("has invalid type, expected {}.", bootstrapapi::SECRET_TYPE_BOOTSTRAP_TOKEN));
        return Ok((None, false));
    }

    let ts = bootstrapsecretutil::get_data(&secret, bootstrapapi::BOOTSTRAP_TOKEN_SECRET_KEY);
    if subtle::constant_time_compare(ts.as_bytes(), token_secret.as_bytes()) != 1 {
        token_errorf(&secret, &format!("has invalid value for key {}.", bootstrapapi::BOOTSTRAP_TOKEN_SECRET_KEY));
        return Ok((None, false));
    }

    let id = bootstrapsecretutil::get_data(&secret, bootstrapapi::BOOTSTRAP_TOKEN_ID_KEY);
    if id != token_id {
        token_errorf(&secret, &format!("has invalid value for key {}.", bootstrapapi::BOOTSTRAP_TOKEN_ID_KEY));
        return Ok((None, false));
    }

    if bootstrapsecretutil::has_expired(&secret, time::now()) {
        // 日志记录在 is_secret_expired 方法中完成。
        return Ok((None, false));
    }

    if bootstrapsecretutil::get_data(&secret, bootstrapapi::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION) != "true" {
        token_errorf(&secret, &format!("not marked {}=true.", bootstrapapi::BOOTSTRAP_TOKEN_USAGE_AUTHENTICATION));
        return Ok((None, false));
    }

    let groups = match bootstrapsecretutil::get_groups(&secret) {
        Ok(g) => g,
        Err(err) => {
            token_errorf(&secret, &format!("has invalid value for key {}: {:?}.", bootstrapapi::BOOTSTRAP_TOKEN_EXTRA_GROUPS_KEY, err));
            return Ok((None, false));
        }
    };

    Ok((
        Some(authenticator::Response {
            user: user::DefaultInfo {
                name: format!("{}{}", bootstrapapi::BOOTSTRAP_USER_PREFIX, id),
                groups,
            },
        }),
        true,
    ))
}
