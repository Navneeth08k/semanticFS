pub fn issue_token(user: &str) -> String {
    format!("tok_{}_v1", user)
}

pub fn validate_token(token: &str) -> bool {
    token.starts_with("tok_") && token.ends_with("_v1")
}
