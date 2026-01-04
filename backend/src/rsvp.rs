use anyhow::anyhow;
use axum::{extract::State, Json};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{
    auth::get_current_session,
    error::AppError,
    models::{Guest, Rsvp, RsvpAttendee, SessionType},
    schemas::{AttendeeResponse, RsvpResponse, RsvpStatusResponse, SubmitRsvpRequest},
    Result, ValidatedRequest,
};

/// Helper to get guest from current session.
async fn get_guest_from_session(pool: &PgPool, cookies: &Cookies) -> Result<Guest> {
    let session = get_current_session(pool, cookies)
        .await
        .ok_or(AppError::Unauthorized)?;

    if session.get_session_type() != Some(SessionType::Guest) {
        return Err(AppError::Unauthorized);
    }

    let guest_id = session
        .guest_id
        .ok_or_else(|| AppError::Internal(anyhow!("Guest session missing guest_id")))?;

    sqlx::query_as::<_, Guest>("SELECT * FROM guests WHERE id = $1")
        .bind(guest_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Guest not found".into()))
}

/// GET /rsvp - Get RSVP status for current guest.
#[utoipa::path(
    get,
    path = "/rsvp",
    responses(
        (status = 200, body = RsvpStatusResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_rsvp_status(
    State(pool): State<PgPool>,
    cookies: Cookies,
) -> Result<Json<RsvpStatusResponse>> {
    let guest = get_guest_from_session(&pool, &cookies).await?;

    let rsvp = sqlx::query_as::<_, Rsvp>("SELECT * FROM rsvps WHERE guest_id = $1")
        .bind(guest.id)
        .fetch_optional(&pool)
        .await?;

    let response = if let Some(rsvp) = rsvp {
        let attendees = sqlx::query_as::<_, RsvpAttendee>(
            "SELECT * FROM rsvp_attendees WHERE rsvp_id = $1 ORDER BY is_primary DESC, name",
        )
        .bind(rsvp.id)
        .fetch_all(&pool)
        .await?;

        RsvpStatusResponse {
            has_responded: true,
            party_size: guest.party_size,
            guest_name: guest.name,
            rsvp: Some(RsvpResponse {
                id: rsvp.id,
                guest_id: rsvp.guest_id,
                responded_at: rsvp.responded_at.to_rfc3339(),
                attendees: attendees
                    .into_iter()
                    .map(|a| AttendeeResponse {
                        id: a.id,
                        name: a.name,
                        is_attending: a.is_attending,
                        meal_preference: a.meal_preference,
                        dietary_restrictions: a.dietary_restrictions,
                        is_primary: a.is_primary,
                    })
                    .collect(),
            }),
        }
    } else {
        RsvpStatusResponse {
            has_responded: false,
            party_size: guest.party_size,
            guest_name: guest.name,
            rsvp: None,
        }
    };

    Ok(Json(response))
}

/// POST /rsvp - Submit or update RSVP.
#[utoipa::path(
    post,
    path = "/rsvp",
    request_body = SubmitRsvpRequest,
    responses(
        (status = 200, body = RsvpResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn submit_rsvp(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Json(input): Json<SubmitRsvpRequest>,
) -> Result<Json<RsvpResponse>> {
    input.validate_request().map_err(AppError::validation)?;

    let guest = get_guest_from_session(&pool, &cookies).await?;

    // Validate attendee count against party_size
    if input.attendees.len() > guest.party_size as usize {
        return Err(AppError::BadRequest(format!(
            "Cannot have more than {} attendees",
            guest.party_size
        )));
    }

    // Ensure exactly one primary attendee
    let primary_count = input.attendees.iter().filter(|a| a.is_primary).count();
    if primary_count != 1 {
        return Err(AppError::BadRequest(
            "Exactly one attendee must be marked as primary".into(),
        ));
    }

    // Validate meal preferences
    let valid_meals = ["beef", "chicken", "fish", "vegetarian", "vegan"];
    for att in &input.attendees {
        if let Some(ref meal) = att.meal_preference {
            if !valid_meals.contains(&meal.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Invalid meal preference: {}",
                    meal
                )));
            }
        }
    }

    // Start transaction
    let mut tx = pool.begin().await?;

    // Delete existing RSVP if any (cascade deletes attendees)
    sqlx::query("DELETE FROM rsvps WHERE guest_id = $1")
        .bind(guest.id)
        .execute(&mut *tx)
        .await?;

    // Create new RSVP
    let rsvp = sqlx::query_as::<_, Rsvp>("INSERT INTO rsvps (guest_id) VALUES ($1) RETURNING *")
        .bind(guest.id)
        .fetch_one(&mut *tx)
        .await?;

    // Insert attendees
    let mut attendees = Vec::new();
    for att in input.attendees {
        let attendee = sqlx::query_as::<_, RsvpAttendee>(
            r#"
            INSERT INTO rsvp_attendees (rsvp_id, name, is_attending, meal_preference, dietary_restrictions, is_primary)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(rsvp.id)
        .bind(&att.name)
        .bind(att.is_attending)
        .bind(&att.meal_preference)
        .bind(&att.dietary_restrictions)
        .bind(att.is_primary)
        .fetch_one(&mut *tx)
        .await?;

        attendees.push(AttendeeResponse {
            id: attendee.id,
            name: attendee.name,
            is_attending: attendee.is_attending,
            meal_preference: attendee.meal_preference,
            dietary_restrictions: attendee.dietary_restrictions,
            is_primary: attendee.is_primary,
        });
    }

    tx.commit().await?;

    Ok(Json(RsvpResponse {
        id: rsvp.id,
        guest_id: rsvp.guest_id,
        responded_at: rsvp.responded_at.to_rfc3339(),
        attendees,
    }))
}
