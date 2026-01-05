//! Admin handlers for guest, event, and settings management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;
use sqlx::PgPool;
use tower_cookies::Cookies;
use uuid::Uuid;

use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};

use crate::{
    auth::{get_current_session, hash_password},
    error::AppError,
    models::{Admin, Event, Guest, Session, SessionType},
    schemas::{
        AdminEventResponse, AdminEventsListResponse, AdminGuestResponse, AdminGuestsListResponse,
        AdminRsvpSummary, ChangePasswordRequest, ChangePasswordResponse, CreateEventRequest,
        CreateGuestRequest, CreateGuestResponse, DashboardStatsResponse, GenerateCodeResponse,
        RecentRsvp, UpdateEventRequest, UpdateGuestRequest,
    },
    Result, ValidatedRequest,
};

/// Verify the request is from an authenticated admin.
async fn require_admin(pool: &PgPool, cookies: &Cookies) -> Result<()> {
    let session = get_current_session(pool, cookies)
        .await
        .ok_or(AppError::Unauthorized)?;

    if session.get_session_type() != Some(SessionType::Admin) {
        return Err(AppError::Unauthorized);
    }

    Ok(())
}

/// Generate a random invite code (6 alphanumeric characters).
fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed ambiguous chars
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Helper to build AdminGuestResponse with RSVP info.
async fn build_guest_response(pool: &PgPool, guest: &Guest) -> Result<AdminGuestResponse> {
    // Get invite code
    let invite_code: Option<String> = sqlx::query_scalar(
        "SELECT code FROM invite_codes WHERE guest_id = $1 AND code_type = 'guest'",
    )
    .bind(guest.id)
    .fetch_optional(pool)
    .await?;

    // Get RSVP status
    let rsvp_row: Option<(chrono::DateTime<chrono::Utc>,)> =
        sqlx::query_as("SELECT responded_at FROM rsvps WHERE guest_id = $1")
            .bind(guest.id)
            .fetch_optional(pool)
            .await?;

    let (has_responded, responded_at, attending_count, not_attending_count) =
        if let Some((responded,)) = rsvp_row {
            // Get attendee counts
            let counts: (i64, i64) = sqlx::query_as(
                r#"
            SELECT
                COALESCE(SUM(CASE WHEN ra.is_attending THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN NOT ra.is_attending THEN 1 ELSE 0 END), 0)
            FROM rsvp_attendees ra
            JOIN rsvps r ON ra.rsvp_id = r.id
            WHERE r.guest_id = $1
            "#,
            )
            .bind(guest.id)
            .fetch_one(pool)
            .await?;

            (
                true,
                Some(responded.to_rfc3339()),
                counts.0 as i32,
                counts.1 as i32,
            )
        } else {
            (false, None, 0, 0)
        };

    Ok(AdminGuestResponse {
        id: guest.id,
        name: guest.name.clone(),
        party_size: guest.party_size,
        invite_code,
        rsvp: AdminRsvpSummary {
            has_responded,
            responded_at,
            attending_count,
            not_attending_count,
        },
        created_at: guest.created_at.to_rfc3339(),
    })
}

/// GET /admin/guests - List all guests with their invite codes and RSVP status.
#[utoipa::path(
    get,
    path = "/admin/guests",
    responses(
        (status = 200, body = AdminGuestsListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_guests(
    State(pool): State<PgPool>,
    cookies: Cookies,
) -> Result<Json<AdminGuestsListResponse>> {
    require_admin(&pool, &cookies).await?;

    let guests = sqlx::query_as::<_, Guest>("SELECT * FROM guests ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await?;

    let total = guests.len() as i64;

    let mut guest_responses = Vec::with_capacity(guests.len());
    for guest in &guests {
        guest_responses.push(build_guest_response(&pool, guest).await?);
    }

    Ok(Json(AdminGuestsListResponse {
        guests: guest_responses,
        total,
    }))
}

/// POST /admin/guests - Create a new guest with an invite code.
#[utoipa::path(
    post,
    path = "/admin/guests",
    request_body = CreateGuestRequest,
    responses(
        (status = 201, body = CreateGuestResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_guest(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<CreateGuestRequest>,
) -> Result<(StatusCode, Json<CreateGuestResponse>)> {
    require_admin(&pool, &cookies).await?;
    input.validate_request().map_err(AppError::validation)?;

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create guest
    let guest = sqlx::query_as::<_, Guest>(
        "INSERT INTO guests (name, party_size) VALUES ($1, $2) RETURNING *",
    )
    .bind(&input.name)
    .bind(input.party_size)
    .fetch_one(&mut *tx)
    .await?;

    // Generate unique invite code
    let invite_code = loop {
        let code = generate_invite_code();
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM invite_codes WHERE code = $1)")
                .bind(&code)
                .fetch_one(&mut *tx)
                .await?;
        if !exists {
            break code;
        }
    };

    // Create invite code
    sqlx::query("INSERT INTO invite_codes (code, code_type, guest_id) VALUES ($1, 'guest', $2)")
        .bind(&invite_code)
        .bind(guest.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateGuestResponse {
            id: guest.id,
            name: guest.name,
            party_size: guest.party_size,
            invite_code,
        }),
    ))
}

/// PUT /admin/guests/:id - Update a guest.
#[utoipa::path(
    put,
    path = "/admin/guests/{id}",
    params(("id" = Uuid, Path, description = "Guest ID")),
    request_body = UpdateGuestRequest,
    responses(
        (status = 200, body = AdminGuestResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Guest not found")
    )
)]
pub async fn update_guest(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateGuestRequest>,
) -> Result<Json<AdminGuestResponse>> {
    require_admin(&pool, &cookies).await?;
    input.validate_request().map_err(AppError::validation)?;

    let guest = sqlx::query_as::<_, Guest>(
        "UPDATE guests SET name = $1, party_size = $2 WHERE id = $3 RETURNING *",
    )
    .bind(&input.name)
    .bind(input.party_size)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Guest not found".into()))?;

    let response = build_guest_response(&pool, &guest).await?;
    Ok(Json(response))
}

/// DELETE /admin/guests/:id - Delete a guest and their invite codes.
#[utoipa::path(
    delete,
    path = "/admin/guests/{id}",
    params(("id" = Uuid, Path, description = "Guest ID")),
    responses(
        (status = 204, description = "Guest deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Guest not found")
    )
)]
pub async fn delete_guest(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_admin(&pool, &cookies).await?;

    let result = sqlx::query("DELETE FROM guests WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Guest not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /admin/guests/:id/code - Generate a new invite code for a guest.
#[utoipa::path(
    post,
    path = "/admin/guests/{id}/code",
    params(("id" = Uuid, Path, description = "Guest ID")),
    responses(
        (status = 200, body = GenerateCodeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Guest not found")
    )
)]
pub async fn regenerate_code(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Path(id): Path<Uuid>,
) -> Result<Json<GenerateCodeResponse>> {
    require_admin(&pool, &cookies).await?;

    // Verify guest exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM guests WHERE id = $1)")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    if !exists {
        return Err(AppError::NotFound("Guest not found".into()));
    }

    let mut tx = pool.begin().await?;

    // Delete old codes for this guest
    sqlx::query("DELETE FROM invite_codes WHERE guest_id = $1 AND code_type = 'guest'")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Generate new unique code
    let invite_code = loop {
        let code = generate_invite_code();
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM invite_codes WHERE code = $1)")
                .bind(&code)
                .fetch_one(&mut *tx)
                .await?;
        if !exists {
            break code;
        }
    };

    // Create new invite code
    sqlx::query("INSERT INTO invite_codes (code, code_type, guest_id) VALUES ($1, 'guest', $2)")
        .bind(&invite_code)
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(GenerateCodeResponse { invite_code }))
}

/// GET /admin/dashboard - Get dashboard statistics.
#[utoipa::path(
    get,
    path = "/admin/dashboard",
    responses(
        (status = 200, body = DashboardStatsResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_dashboard_stats(
    State(pool): State<PgPool>,
    cookies: Cookies,
) -> Result<Json<DashboardStatsResponse>> {
    require_admin(&pool, &cookies).await?;

    // Get total guests and total expected attendees
    let guest_stats: (i64, i64) =
        sqlx::query_as("SELECT COUNT(*), COALESCE(SUM(party_size), 0) FROM guests")
            .fetch_one(&pool)
            .await?;

    let total_guests = guest_stats.0;
    let total_expected = guest_stats.1;

    // Get RSVP counts
    let rsvp_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rsvps")
        .fetch_one(&pool)
        .await?;

    // Get attendee counts
    let attendee_stats: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN is_attending THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN NOT is_attending THEN 1 ELSE 0 END), 0)
        FROM rsvp_attendees
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let attending_count = attendee_stats.0;
    let not_attending_count = attendee_stats.1;

    // Get recent RSVPs (last 5)
    let recent_rsvps: Vec<(String, chrono::DateTime<chrono::Utc>, i64, i64)> = sqlx::query_as(
        r#"
        SELECT
            g.name,
            r.responded_at,
            COALESCE(SUM(CASE WHEN ra.is_attending THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN NOT ra.is_attending THEN 1 ELSE 0 END), 0)
        FROM rsvps r
        JOIN guests g ON r.guest_id = g.id
        LEFT JOIN rsvp_attendees ra ON ra.rsvp_id = r.id
        GROUP BY g.name, r.responded_at
        ORDER BY r.responded_at DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let recent_rsvps = recent_rsvps
        .into_iter()
        .map(
            |(name, responded_at, attending, not_attending)| RecentRsvp {
                guest_name: name,
                responded_at: responded_at.to_rfc3339(),
                attending_count: attending as i32,
                not_attending_count: not_attending as i32,
            },
        )
        .collect();

    Ok(Json(DashboardStatsResponse {
        total_guests,
        total_expected_attendees: total_expected,
        rsvp_count,
        pending_rsvps: total_guests - rsvp_count,
        attending_count,
        not_attending_count,
        recent_rsvps,
    }))
}

// ============================================================================
// Event Management
// ============================================================================

/// Helper to convert Event model to AdminEventResponse.
fn event_to_response(event: &Event) -> AdminEventResponse {
    AdminEventResponse {
        id: event.id,
        name: event.name.clone(),
        event_type: event.event_type.clone(),
        event_date: event.event_date.to_string(),
        event_time: event.event_time.format("%H:%M").to_string(),
        location_name: event.location_name.clone(),
        location_address: event.location_address.clone(),
        description: event.description.clone(),
        display_order: event.display_order,
    }
}

/// GET /admin/events - List all events for admin.
#[utoipa::path(
    get,
    path = "/admin/events",
    responses(
        (status = 200, body = AdminEventsListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_admin_events(
    State(pool): State<PgPool>,
    cookies: Cookies,
) -> Result<Json<AdminEventsListResponse>> {
    require_admin(&pool, &cookies).await?;

    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events ORDER BY display_order, event_date, event_time",
    )
    .fetch_all(&pool)
    .await?;

    let events: Vec<AdminEventResponse> = events.iter().map(event_to_response).collect();

    Ok(Json(AdminEventsListResponse { events }))
}

/// POST /admin/events - Create a new event.
#[utoipa::path(
    post,
    path = "/admin/events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, body = AdminEventResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_event(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<AdminEventResponse>)> {
    require_admin(&pool, &cookies).await?;
    input.validate_request().map_err(AppError::validation)?;

    // Parse date and time
    let event_date = chrono::NaiveDate::parse_from_str(&input.event_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format. Use YYYY-MM-DD".into()))?;
    let event_time = chrono::NaiveTime::parse_from_str(&input.event_time, "%H:%M")
        .map_err(|_| AppError::BadRequest("Invalid time format. Use HH:MM".into()))?;

    let event = sqlx::query_as::<_, Event>(
        r#"
        INSERT INTO events (name, event_type, event_date, event_time, location_name, location_address, description, display_order)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(&input.name)
    .bind(&input.event_type)
    .bind(event_date)
    .bind(event_time)
    .bind(&input.location_name)
    .bind(&input.location_address)
    .bind(&input.description)
    .bind(input.display_order)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(event_to_response(&event))))
}

/// PUT /admin/events/:id - Update an event.
#[utoipa::path(
    put,
    path = "/admin/events/{id}",
    params(("id" = Uuid, Path, description = "Event ID")),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, body = AdminEventResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Event not found")
    )
)]
pub async fn update_event(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateEventRequest>,
) -> Result<Json<AdminEventResponse>> {
    require_admin(&pool, &cookies).await?;
    input.validate_request().map_err(AppError::validation)?;

    // Parse date and time
    let event_date = chrono::NaiveDate::parse_from_str(&input.event_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format. Use YYYY-MM-DD".into()))?;
    let event_time = chrono::NaiveTime::parse_from_str(&input.event_time, "%H:%M")
        .map_err(|_| AppError::BadRequest("Invalid time format. Use HH:MM".into()))?;

    let event = sqlx::query_as::<_, Event>(
        r#"
        UPDATE events
        SET name = $1, event_type = $2, event_date = $3, event_time = $4,
            location_name = $5, location_address = $6, description = $7, display_order = $8
        WHERE id = $9
        RETURNING *
        "#,
    )
    .bind(&input.name)
    .bind(&input.event_type)
    .bind(event_date)
    .bind(event_time)
    .bind(&input.location_name)
    .bind(&input.location_address)
    .bind(&input.description)
    .bind(input.display_order)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Event not found".into()))?;

    Ok(Json(event_to_response(&event)))
}

/// DELETE /admin/events/:id - Delete an event.
#[utoipa::path(
    delete,
    path = "/admin/events/{id}",
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 204, description = "Event deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Event not found")
    )
)]
pub async fn delete_event(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_admin(&pool, &cookies).await?;

    let result = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Event not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Settings
// ============================================================================

/// Get current admin session with admin_id.
async fn get_admin_session(pool: &PgPool, cookies: &Cookies) -> Result<Session> {
    let session = get_current_session(pool, cookies)
        .await
        .ok_or(AppError::Unauthorized)?;

    if session.get_session_type() != Some(SessionType::Admin) {
        return Err(AppError::Unauthorized);
    }

    Ok(session)
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

/// POST /admin/settings/password - Change admin password.
#[utoipa::path(
    post,
    path = "/admin/settings/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, body = ChangePasswordResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized or wrong password")
    )
)]
pub async fn change_password(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordResponse>> {
    let session = get_admin_session(&pool, &cookies).await?;
    input.validate_request().map_err(AppError::validation)?;

    let admin_id = session.admin_id.ok_or_else(|| AppError::Unauthorized)?;

    // Get current admin
    let admin = sqlx::query_as::<_, Admin>("SELECT * FROM admins WHERE id = $1")
        .bind(admin_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify current password
    if !verify_password(&input.current_password, &admin.password_hash) {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }

    // Hash new password
    let new_hash = hash_password(&input.new_password)?;

    // Update password
    sqlx::query("UPDATE admins SET password_hash = $1 WHERE id = $2")
        .bind(&new_hash)
        .bind(admin_id)
        .execute(&pool)
        .await?;

    Ok(Json(ChangePasswordResponse {
        message: "Password changed successfully".into(),
    }))
}
