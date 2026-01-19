pub mod bootstraptokenutil {
    use regex::Regex;
    use once_cell::sync::Lazy;

    // 使用 Lazy 实现延迟初始化的静态正则表达式
    static BOOTSTRAP_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^([a-z0-9]{6})\.([a-z0-9]{16})$").unwrap()
    });

    // ParseToken 尝试从字符串中解析一个有效的 token。
    // 成功时返回 token ID 和 token secret，否则返回错误。
    pub fn parse_token(s: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let captures = BOOTSTRAP_TOKEN_RE.captures(s);
        match captures {
            Some(caps) if caps.len() == 3 => {
                let token_id = caps.get(1).unwrap().as_str().to_string();
                let token_secret = caps.get(2).unwrap().as_str().to_string();
                Ok((token_id, token_secret))
            }
            _ => {
                Err(format!("token [{}] was not of form [{}]", s, api::BOOTSTRAP_TOKEN_PATTERN).into())
            }
        }
    }
}
