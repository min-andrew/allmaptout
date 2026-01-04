use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CodeType, Guest, Session, SessionType},
    schemas::{
        AdminLoginRequest, AdminLoginResponse, SessionResponse, ValidateCodeRequest,
        ValidateCodeResponse,
    },
    Result, ValidatedRequest,
};

const SESSION_COOKIE_NAME: &str = "session";
const SESSION_DURATION_DAYS: i64 = 7;

/// Generate a random session token.
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Hash a password using Argon2.
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal(anyhow!("Failed to hash password")))
}

/// Verify a password against a hash.
fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Create a new session in the database.
async fn create_session(
    pool: &PgPool,
    session_type: SessionType,
    guest_id: Option<Uuid>,
    admin_id: Option<Uuid>,
) -> Result<Session> {
    let token = generate_token();
    let expires_at = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

    let session = sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (token, session_type, guest_id, admin_id, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(&token)
    .bind(session_type.as_str())
    .bind(guest_id)
    .bind(admin_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(session)
}

/// Set the session cookie.
fn set_session_cookie(cookies: &Cookies, token: &str) {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token.to_string());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    // In production, set secure = true
    if std::env::var("RUST_ENV").unwrap_or_default() != "development" {
        cookie.set_secure(true);
    }
    cookies.add(cookie);
}

/// Remove the session cookie.
fn remove_session_cookie(cookies: &Cookies) {
    cookies.remove(Cookie::from(SESSION_COOKIE_NAME));
}

/// Get the current session from cookies.
pub async fn get_current_session(pool: &PgPool, cookies: &Cookies) -> Option<Session> {
    let token = cookies.get(SESSION_COOKIE_NAME)?.value().to_string();

    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE token = $1 AND expires_at > NOW()",
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(session)
}

/// POST /auth/code - Validate an invite code and create a session.
#[utoipa::path(
    post,
    path = "/auth/code",
    request_body = ValidateCodeRequest,
    responses(
        (status = 200, body = ValidateCodeResponse),
        (status = 400, description = "Invalid code")
    )
)]
pub async fn validate_code(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<ValidateCodeRequest>,
) -> Result<Json<ValidateCodeResponse>> {
    input.validate_request().map_err(AppError::validation)?;

    // Look up the invite code
    let invite_code = sqlx::query_as::<_, crate::models::InviteCode>(
        "SELECT * FROM invite_codes WHERE code = $1",
    )
    .bind(&input.code)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid code".into()))?;

    let code_type = invite_code
        .get_code_type()
        .ok_or_else(|| AppError::Internal(anyhow!("Invalid code type in database")))?;

    let (session_type, guest_id, guest_name) = match code_type {
        CodeType::Guest => {
            let guest_id = invite_code
                .guest_id
                .ok_or_else(|| AppError::Internal(anyhow!("Guest code missing guest_id")))?;

            let guest = sqlx::query_as::<_, Guest>("SELECT * FROM guests WHERE id = $1")
                .bind(guest_id)
                .fetch_optional(&pool)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow!("Guest not found")))?;

            (SessionType::Guest, Some(guest_id), Some(guest.name))
        }
        CodeType::Admin => (SessionType::AdminPending, None, None),
    };

    // Create session
    let session = create_session(&pool, session_type.clone(), guest_id, None).await?;
    set_session_cookie(&cookies, &session.token);

    Ok(Json(ValidateCodeResponse {
        session_type: session_type.as_str().to_string(),
        guest_name,
    }))
}

/// POST /auth/admin/login - Admin login with username/password.
#[utoipa::path(
    post,
    path = "/auth/admin/login",
    request_body = AdminLoginRequest,
    responses(
        (status = 200, body = AdminLoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Admin-pending session required")
    )
)]
pub async fn admin_login(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<AdminLoginRequest>,
) -> Result<Json<AdminLoginResponse>> {
    input.validate_request().map_err(AppError::validation)?;

    // Require admin-pending session
    let current_session = get_current_session(&pool, &cookies)
        .await
        .ok_or(AppError::Unauthorized)?;

    if current_session.get_session_type() != Some(SessionType::AdminPending) {
        return Err(AppError::Unauthorized);
    }

    // Look up the admin
    let admin =
        sqlx::query_as::<_, crate::models::Admin>("SELECT * FROM admins WHERE username = $1")
            .bind(&input.username)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::Unauthorized)?;

    // Verify password
    if !verify_password(&input.password, &admin.password_hash) {
        return Err(AppError::Unauthorized);
    }

    // Delete old session
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(current_session.id)
        .execute(&pool)
        .await?;

    // Create full admin session
    let session = create_session(&pool, SessionType::Admin, None, Some(admin.id)).await?;
    set_session_cookie(&cookies, &session.token);

    Ok(Json(AdminLoginResponse {
        username: admin.username,
    }))
}

/// POST /auth/logout - Log out and clear session.
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses((status = 204, description = "Logged out"))
)]
pub async fn logout(State(pool): State<PgPool>, cookies: Cookies) -> Result<impl IntoResponse> {
    if let Some(session) = get_current_session(&pool, &cookies).await {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session.id)
            .execute(&pool)
            .await?;
    }

    remove_session_cookie(&cookies);
    Ok(StatusCode::NO_CONTENT)
}

/// GET /auth/session - Get current session info.
#[utoipa::path(
    get,
    path = "/auth/session",
    responses(
        (status = 200, body = SessionResponse),
        (status = 401, description = "Not logged in")
    )
)]
pub async fn get_session(
    State(pool): State<PgPool>,
    cookies: Cookies,
) -> Result<Json<SessionResponse>> {
    let session = get_current_session(&pool, &cookies)
        .await
        .ok_or(AppError::Unauthorized)?;

    let session_type = session
        .get_session_type()
        .ok_or_else(|| AppError::Internal(anyhow!("Invalid session type")))?;

    let (guest_id, guest_name, admin_id, admin_username) = match session_type {
        SessionType::Guest => {
            let guest_id = session.guest_id;
            let guest_name = if let Some(gid) = guest_id {
                sqlx::query_scalar::<_, String>("SELECT name FROM guests WHERE id = $1")
                    .bind(gid)
                    .fetch_optional(&pool)
                    .await?
            } else {
                None
            };
            (guest_id, guest_name, None, None)
        }
        SessionType::AdminPending => (None, None, None, None),
        SessionType::Admin => {
            let admin_id = session.admin_id;
            let admin_username = if let Some(aid) = admin_id {
                sqlx::query_scalar::<_, String>("SELECT username FROM admins WHERE id = $1")
                    .bind(aid)
                    .fetch_optional(&pool)
                    .await?
            } else {
                None
            };
            (None, None, admin_id, admin_username)
        }
    };

    Ok(Json(SessionResponse {
        session_type: session_type.as_str().to_string(),
        guest_id,
        guest_name,
        admin_id,
        admin_username,
    }))
}
