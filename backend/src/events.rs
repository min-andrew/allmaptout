use axum::{extract::State, Json};
use sqlx::PgPool;

use crate::{
    models::Event,
    schemas::{EventResponse, EventsListResponse},
    Result,
};

/// GET /events - List all events ordered by display_order.
#[utoipa::path(
    get,
    path = "/events",
    responses((status = 200, body = EventsListResponse))
)]
pub async fn list_events(State(pool): State<PgPool>) -> Result<Json<EventsListResponse>> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events ORDER BY display_order, event_date, event_time",
    )
    .fetch_all(&pool)
    .await?;

    let events: Vec<EventResponse> = events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id,
            name: e.name,
            event_type: e.event_type,
            event_date: e.event_date.to_string(),
            event_time: e.event_time.format("%H:%M").to_string(),
            location_name: e.location_name,
            location_address: e.location_address,
            description: e.description,
        })
        .collect();

    Ok(Json(EventsListResponse { events }))
}
