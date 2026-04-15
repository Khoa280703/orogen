use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtUser {
    pub user_id: i32,
    pub email: String,
}

pub fn extract_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string())
}

pub fn validate_token(token: &str) -> Result<JwtUser, String> {
    let secret = extract_jwt_secret();
    let token = token.strip_prefix("Bearer ").unwrap_or(token);

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| format!("Token validation error: {}", e))?;

    Ok(JwtUser {
        user_id: token_data.claims.user_id,
        email: token_data.claims.email,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: i32,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

pub async fn jwt_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    let is_public = path == "/"
        || path == "/health"
        || path == "/api/plans"
        || path.starts_with("/v1/")
        || path.starts_with("/auth/")
        || path == "/auth"
        || path.starts_with("/admin");

    if is_public {
        return Ok(next.run(req).await);
    }

    // Extract Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT token
    match validate_token(auth_header) {
        Ok(user) => {
            // Inject user into request extensions for downstream handlers
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[allow(dead_code)]
pub fn get_user_from_request(req: &Request) -> Option<JwtUser> {
    req.extensions().get::<JwtUser>().cloned()
}

/// Check if JWT token is expired
#[allow(dead_code)]
pub fn is_token_expired(token: &str) -> bool {
    validate_token(token).is_err()
}

/// Get current timestamp for token generation
#[allow(dead_code)]
pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}
