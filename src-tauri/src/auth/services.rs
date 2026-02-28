use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::auth::admin_services;
use crate::auth::models::{LoginData, TokenPair, UserProfile};
use crate::core::error::AppError;
use crate::db::auth_repository;

const ACCESS_TOKEN_LIFETIME_SECONDS: u64 = 2 * 60 * 60;
const REFRESH_TOKEN_LIFETIME_SECONDS: u64 = 7 * 24 * 60 * 60;
const ACCESS_TOKEN_TYPE: &str = "access";
const REFRESH_TOKEN_TYPE: &str = "refresh";
const DEFAULT_JWT_SECRET: &str = "pure-admin-thin-dev-secret-change-me";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    token_type: String,
    iat: u64,
    exp: u64,
}

#[must_use]
pub fn now_millis() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));
    u64::try_from(now.as_millis()).unwrap_or(u64::MAX)
}

#[must_use]
fn now_secs() -> u64 {
    now_millis() / 1000
}

#[must_use]
fn jwt_secret() -> &'static str {
    static JWT_SECRET: OnceLock<String> = OnceLock::new();
    JWT_SECRET
        .get_or_init(|| {
            std::env::var("PURE_ADMIN_JWT_SECRET")
                .ok()
                .filter(|secret| !secret.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_JWT_SECRET.to_string())
        })
        .as_str()
}

#[must_use]
fn default_header() -> Header {
    Header::new(Algorithm::HS256)
}

#[must_use]
fn default_validation() -> Validation {
    Validation::new(Algorithm::HS256)
}

#[must_use]
fn build_claims(subject: &str, token_type: &'static str, issued_at: u64, ttl: u64) -> JwtClaims {
    JwtClaims {
        sub: subject.to_string(),
        token_type: token_type.to_string(),
        iat: issued_at,
        exp: issued_at.saturating_add(ttl),
    }
}

#[must_use]
fn encode_claims(claims: &JwtClaims) -> String {
    encode(
        &default_header(),
        claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .expect("encoding known JWT claims should not fail")
}

#[must_use]
pub fn mint_token_pair(subject: &str) -> TokenPair {
    let issued_at = now_secs();
    let access_claims = build_claims(
        subject,
        ACCESS_TOKEN_TYPE,
        issued_at,
        ACCESS_TOKEN_LIFETIME_SECONDS,
    );
    let refresh_claims = build_claims(
        subject,
        REFRESH_TOKEN_TYPE,
        issued_at,
        REFRESH_TOKEN_LIFETIME_SECONDS,
    );

    TokenPair {
        access_token: encode_claims(&access_claims),
        refresh_token: encode_claims(&refresh_claims),
        expires: access_claims.exp.saturating_mul(1000),
    }
}

pub fn verify_refresh_token(refresh_token: &str) -> Result<String, AppError> {
    let decoded = decode::<JwtClaims>(
        refresh_token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &default_validation(),
    )
    .map_err(|_| AppError::Validation("invalid refreshToken".to_string()))?;

    let claims = decoded.claims;
    if claims.token_type != REFRESH_TOKEN_TYPE || claims.sub.trim().is_empty() {
        return Err(AppError::Validation("invalid refreshToken".to_string()));
    }

    Ok(claims.sub)
}

pub fn resolve_user_profile(username: &str, password: &str) -> Result<UserProfile, AppError> {
    let profile = auth_repository::find_user_profile(username, password)?
        .ok_or_else(|| AppError::Validation("invalid username or password".to_string()))?;

    admin_services::ensure_user_available_with_message(
        &profile.username,
        "invalid username or password",
        now_millis(),
    )?;

    Ok(profile)
}

#[must_use]
pub fn build_login_data(profile: UserProfile) -> LoginData {
    let token = mint_token_pair(&profile.username);

    LoginData {
        avatar: profile.avatar,
        username: profile.username,
        nickname: profile.nickname,
        roles: profile.roles,
        permissions: profile.permissions,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires: token.expires,
    }
}

pub fn build_async_routes() -> Result<Vec<Value>, AppError> {
    auth_repository::find_async_routes()
}
