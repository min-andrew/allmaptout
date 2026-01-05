use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use http::{
    header::{HeaderName, HeaderValue},
    Method,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_cookies::CookieManagerLayer;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::{KeyExtractor, SmartIpKeyExtractor},
};
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, Span};

pub mod admin;
pub mod auth;
pub mod config;
pub mod error;
pub mod events;
pub mod models;
pub mod rsvp;
pub mod schemas;

pub use error::{AppError, Result};
pub use schemas::ValidatedRequest;

/// Returns true if the request has IP headers (external traffic from load balancer)
fn has_ip_headers(req: &Request) -> bool {
    let headers = req.headers();
    headers.contains_key("x-forwarded-for") || headers.contains_key("x-real-ip")
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct Health {
    pub status: String,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = Health)))]
pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok".into(),
    })
}

fn cors_layer() -> CorsLayer {
    let is_dev = std::env::var("RUST_ENV").unwrap_or_default() == "development";

    let origin = if is_dev {
        "http://localhost:3000".to_string()
    } else {
        std::env::var("CORS_ORIGIN").expect("CORS_ORIGIN must be set in production")
    };

    CorsLayer::new()
        .allow_origin(origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_credentials(true)
}

pub fn create_router(pool: PgPool) -> Router {
    create_router_with_rate_limit(pool, true)
}

pub fn create_router_with_rate_limit(pool: PgPool, enable_rate_limit: bool) -> Router {
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    // Middleware that only applies rate limiting to external requests (with IP headers)
    let rate_limit_middleware = {
        let config = governor_config.clone();
        let enabled = enable_rate_limit;
        middleware::from_fn(move |req: Request, next: Next| {
            let config = config.clone();
            async move {
                // Skip rate limiting for internal requests (no IP headers)
                // or if rate limiting is disabled
                if !enabled || !has_ip_headers(&req) {
                    return next.run(req).await;
                }

                // Apply rate limiting for external requests
                let key = match SmartIpKeyExtractor.extract(&req) {
                    Ok(key) => key,
                    Err(_) => return next.run(req).await, // Can't extract key, allow through
                };

                match config.limiter().check_key(&key) {
                    Ok(_) => next.run(req).await,
                    Err(_) => Response::builder()
                        .status(http::StatusCode::TOO_MANY_REQUESTS)
                        .body(Body::from("Too many requests"))
                        .unwrap(),
                }
            }
        })
    };

    // Configure request/response logging
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let client_ip = request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
                .or_else(|| {
                    request
                        .headers()
                        .get("x-real-ip")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "internal".to_string());

            tracing::info_span!(
                "request",
                method = %request.method(),
                path = %request.uri().path(),
                client_ip = %client_ip,
            )
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Millis),
        )
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                tracing::error!(
                    latency_ms = latency.as_millis(),
                    error = %error,
                    "request failed"
                );
            },
        );

    Router::new()
        .route("/health", get(health))
        .route("/auth/code", post(auth::validate_code))
        .route("/auth/admin/login", post(auth::admin_login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/session", get(auth::get_session))
        .route("/events", get(events::list_events))
        .route("/rsvp", get(rsvp::get_rsvp_status))
        .route("/rsvp", post(rsvp::submit_rsvp))
        // Admin routes
        .route("/admin/dashboard", get(admin::get_dashboard_stats))
        .route("/admin/guests", get(admin::list_guests))
        .route("/admin/guests", post(admin::create_guest))
        .route("/admin/guests/:id", put(admin::update_guest))
        .route("/admin/guests/:id", delete(admin::delete_guest))
        .route("/admin/guests/:id/code", post(admin::regenerate_code))
        .route("/admin/events", get(admin::list_admin_events))
        .route("/admin/events", post(admin::create_event))
        .route("/admin/events/:id", put(admin::update_event))
        .route("/admin/events/:id", delete(admin::delete_event))
        .route("/admin/settings/password", post(admin::change_password))
        .with_state(pool)
        .layer(CookieManagerLayer::new())
        .layer(rate_limit_middleware)
        .layer(trace_layer)
        .layer(cors_layer())
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    // ============================================================================
    // Test Utilities
    // ============================================================================

    async fn test_pool() -> PgPool {
        dotenvy::dotenv().ok();
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    fn test_server(pool: PgPool) -> TestServer {
        std::env::set_var("RUST_ENV", "development");
        TestServer::new(create_router_with_rate_limit(pool, false)).unwrap()
    }

    /// Test fixture helper - creates a guest and returns (guest_id, invite_code)
    async fn create_test_guest(pool: &PgPool, name: &str, party_size: i32) -> (Uuid, String) {
        let guest_id: Uuid = sqlx::query_scalar(
            "INSERT INTO guests (name, party_size) VALUES ($1, $2) RETURNING id",
        )
        .bind(name)
        .bind(party_size)
        .fetch_one(pool)
        .await
        .unwrap();

        let code = format!("TEST{:02X}", rand::random::<u16>());
        sqlx::query(
            "INSERT INTO invite_codes (code, code_type, guest_id) VALUES ($1, 'guest', $2)",
        )
        .bind(&code)
        .bind(guest_id)
        .execute(pool)
        .await
        .unwrap();

        (guest_id, code)
    }

    /// Test fixture helper - creates an admin and returns (admin_id, admin_code)
    async fn create_test_admin(pool: &PgPool, username: &str, password: &str) -> (Uuid, String) {
        let password_hash = auth::hash_password(password).unwrap();
        let admin_id: Uuid = sqlx::query_scalar(
            "INSERT INTO admins (username, password_hash) VALUES ($1, $2) RETURNING id",
        )
        .bind(username)
        .bind(&password_hash)
        .fetch_one(pool)
        .await
        .unwrap();

        let code = format!("ADM{:03X}", rand::random::<u16>());
        sqlx::query("INSERT INTO invite_codes (code, code_type) VALUES ($1, 'admin')")
            .bind(&code)
            .execute(pool)
            .await
            .unwrap();

        (admin_id, code)
    }

    /// Test fixture helper - creates an event
    async fn create_test_event(pool: &PgPool, name: &str, order: i32) -> Uuid {
        sqlx::query_scalar(
            r#"INSERT INTO events (name, event_type, event_date, event_time, location_name, location_address, display_order)
               VALUES ($1, 'ceremony', '2025-06-15', '14:00', 'Test Venue', '123 Test St', $2)
               RETURNING id"#,
        )
        .bind(name)
        .bind(order)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Cleanup helper - removes test data by pattern
    async fn cleanup_test_data(pool: &PgPool, prefix: &str) {
        // Delete in correct order due to foreign keys
        sqlx::query("DELETE FROM sessions WHERE guest_id IN (SELECT id FROM guests WHERE name LIKE $1) OR admin_id IN (SELECT id FROM admins WHERE username LIKE $1)")
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM rsvp_attendees WHERE rsvp_id IN (SELECT id FROM rsvps WHERE guest_id IN (SELECT id FROM guests WHERE name LIKE $1))")
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
        sqlx::query(
            "DELETE FROM rsvps WHERE guest_id IN (SELECT id FROM guests WHERE name LIKE $1)",
        )
        .bind(format!("{}%", prefix))
        .execute(pool)
        .await
        .ok();
        sqlx::query("DELETE FROM invite_codes WHERE code LIKE $1 OR guest_id IN (SELECT id FROM guests WHERE name LIKE $2)")
            .bind(format!("{}%", prefix))
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM guests WHERE name LIKE $1")
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM admins WHERE username LIKE $1")
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM events WHERE name LIKE $1")
            .bind(format!("{}%", prefix))
            .execute(pool)
            .await
            .ok();
    }

    // ============================================================================
    // Health Tests
    // ============================================================================

    #[tokio::test]
    async fn health_returns_ok() {
        let pool = test_pool().await;
        let server = test_server(pool);
        let response = server.get("/health").await;
        response.assert_status_ok();
        response.assert_json(&json!({"status": "ok"}));
    }

    // ============================================================================
    // Auth Tests
    // ============================================================================

    #[tokio::test]
    async fn validate_guest_code_creates_session() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_VC").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_VC", 2).await;
        let server = test_server(pool.clone());

        let response = server.post("/auth/code").json(&json!({"code": code})).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["session_type"], "guest");
        assert_eq!(body["guest_name"], "TestGuest_VC");

        // Verify session cookie is set
        assert!(response.maybe_cookie("session").is_some());

        cleanup_test_data(&pool, "TestGuest_VC").await;
    }

    #[tokio::test]
    async fn validate_admin_code_creates_pending_session() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_VAC").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_VAC", "password123").await;
        let server = test_server(pool.clone());

        let response = server.post("/auth/code").json(&json!({"code": code})).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["session_type"], "admin_pending");
        assert!(body["guest_name"].is_null());

        cleanup_test_data(&pool, "TestAdmin_VAC").await;
    }

    #[tokio::test]
    async fn validate_invalid_code_returns_400() {
        let pool = test_pool().await;
        let server = test_server(pool);

        let response = server
            .post("/auth/code")
            .json(&json!({"code": "INVALID_CODE_XYZ"}))
            .await;

        response.assert_status(http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn admin_login_requires_pending_session() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_ALRPS").await;
        create_test_admin(&pool, "TestAdmin_ALRPS", "password123").await;
        let server = test_server(pool.clone());

        // Try to login without any session
        let response = server
            .post("/auth/admin/login")
            .json(&json!({"username": "TestAdmin_ALRPS", "password": "password123"}))
            .await;

        response.assert_status(http::StatusCode::UNAUTHORIZED);

        cleanup_test_data(&pool, "TestAdmin_ALRPS").await;
    }

    #[tokio::test]
    async fn admin_login_full_flow() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_ALFF").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_ALFF", "password123").await;
        let server = test_server(pool.clone());

        // Step 1: Validate admin code
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        response.assert_status_ok();
        let session_cookie = response.cookie("session");

        // Step 2: Login with credentials
        let response = server
            .post("/auth/admin/login")
            .add_cookie(session_cookie)
            .json(&json!({"username": "TestAdmin_ALFF", "password": "password123"}))
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["username"], "TestAdmin_ALFF");

        cleanup_test_data(&pool, "TestAdmin_ALFF").await;
    }

    #[tokio::test]
    async fn admin_login_wrong_password_returns_401() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_WP").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_WP", "password123").await;
        let server = test_server(pool.clone());

        // Get pending session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Try wrong password
        let response = server
            .post("/auth/admin/login")
            .add_cookie(session_cookie)
            .json(&json!({"username": "TestAdmin_WP", "password": "wrongpassword"}))
            .await;

        response.assert_status(http::StatusCode::UNAUTHORIZED);

        cleanup_test_data(&pool, "TestAdmin_WP").await;
    }

    #[tokio::test]
    async fn get_session_returns_guest_info() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_GS").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_GS", 3).await;
        let server = test_server(pool.clone());

        // Create session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Get session info
        let response = server.get("/auth/session").add_cookie(session_cookie).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["session_type"], "guest");
        assert_eq!(body["guest_name"], "TestGuest_GS");
        assert!(body["guest_id"].is_string());

        cleanup_test_data(&pool, "TestGuest_GS").await;
    }

    #[tokio::test]
    async fn get_session_without_cookie_returns_401() {
        let pool = test_pool().await;
        let server = test_server(pool);

        let response = server.get("/auth/session").await;
        response.assert_status(http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn logout_clears_session() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_LO").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_LO", 1).await;
        let server = test_server(pool.clone());

        // Create session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Logout
        let response = server
            .post("/auth/logout")
            .add_cookie(session_cookie.clone())
            .await;
        response.assert_status(http::StatusCode::NO_CONTENT);

        // Verify session is invalid
        let response = server.get("/auth/session").add_cookie(session_cookie).await;
        response.assert_status(http::StatusCode::UNAUTHORIZED);

        cleanup_test_data(&pool, "TestGuest_LO").await;
    }

    // ============================================================================
    // Events Tests
    // ============================================================================

    #[tokio::test]
    async fn list_events_returns_ordered() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestEvent_LE").await;

        // Create events in non-sequential order
        create_test_event(&pool, "TestEvent_LE_Second", 2).await;
        create_test_event(&pool, "TestEvent_LE_First", 1).await;

        let server = test_server(pool.clone());
        let response = server.get("/events").await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        let events = body["events"].as_array().unwrap();

        // Find our test events and verify order
        let test_events: Vec<_> = events
            .iter()
            .filter(|e| e["name"].as_str().unwrap().starts_with("TestEvent_LE"))
            .collect();

        assert!(test_events.len() >= 2);
        // First should come before Second based on display_order
        let first_idx = test_events
            .iter()
            .position(|e| e["name"] == "TestEvent_LE_First");
        let second_idx = test_events
            .iter()
            .position(|e| e["name"] == "TestEvent_LE_Second");
        assert!(first_idx < second_idx);

        cleanup_test_data(&pool, "TestEvent_LE").await;
    }

    // ============================================================================
    // RSVP Tests
    // ============================================================================

    #[tokio::test]
    async fn get_rsvp_status_for_new_guest() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_RSVP").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_RSVP", 2).await;
        let server = test_server(pool.clone());

        // Create session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Get RSVP status
        let response = server.get("/rsvp").add_cookie(session_cookie).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(!body["has_responded"].as_bool().unwrap());
        assert_eq!(body["party_size"], 2);
        assert_eq!(body["guest_name"], "TestGuest_RSVP");
        assert!(body["rsvp"].is_null());

        cleanup_test_data(&pool, "TestGuest_RSVP").await;
    }

    #[tokio::test]
    async fn submit_rsvp_success() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_SR").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_SR", 2).await;
        let server = test_server(pool.clone());

        // Create session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Submit RSVP
        let response = server
            .post("/rsvp")
            .add_cookie(session_cookie.clone())
            .json(&json!({
                "attendees": [
                    {
                        "name": "Primary Guest",
                        "is_attending": true,
                        "meal_preference": "chicken",
                        "dietary_restrictions": null,
                        "is_primary": true
                    },
                    {
                        "name": "Plus One",
                        "is_attending": true,
                        "meal_preference": "vegetarian",
                        "dietary_restrictions": "No nuts",
                        "is_primary": false
                    }
                ]
            }))
            .await;

        response.assert_status_ok();

        // Verify RSVP was saved
        let response = server.get("/rsvp").add_cookie(session_cookie).await;
        let body: serde_json::Value = response.json();
        assert!(body["has_responded"].as_bool().unwrap());
        let attendees = body["rsvp"]["attendees"].as_array().unwrap();
        assert_eq!(attendees.len(), 2);

        cleanup_test_data(&pool, "TestGuest_SR").await;
    }

    #[tokio::test]
    async fn submit_rsvp_exceeds_party_size() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_EPS").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_EPS", 1).await;
        let server = test_server(pool.clone());

        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Try to submit with 2 attendees but party size is 1
        let response = server
            .post("/rsvp")
            .add_cookie(session_cookie)
            .json(&json!({
                "attendees": [
                    {"name": "Guest 1", "is_attending": true, "is_primary": true},
                    {"name": "Guest 2", "is_attending": true, "is_primary": false}
                ]
            }))
            .await;

        response.assert_status(http::StatusCode::BAD_REQUEST);

        cleanup_test_data(&pool, "TestGuest_EPS").await;
    }

    #[tokio::test]
    async fn submit_rsvp_requires_primary() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_RP").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_RP", 2).await;
        let server = test_server(pool.clone());

        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // No primary attendee
        let response = server
            .post("/rsvp")
            .add_cookie(session_cookie)
            .json(&json!({
                "attendees": [
                    {"name": "Guest 1", "is_attending": true, "is_primary": false}
                ]
            }))
            .await;

        response.assert_status(http::StatusCode::BAD_REQUEST);

        cleanup_test_data(&pool, "TestGuest_RP").await;
    }

    #[tokio::test]
    async fn submit_rsvp_validates_meal_preference() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestGuest_VMP").await;

        let (_, code) = create_test_guest(&pool, "TestGuest_VMP", 1).await;
        let server = test_server(pool.clone());

        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Invalid meal preference
        let response = server
            .post("/rsvp")
            .add_cookie(session_cookie)
            .json(&json!({
                "attendees": [
                    {"name": "Guest", "is_attending": true, "meal_preference": "pizza", "is_primary": true}
                ]
            }))
            .await;

        response.assert_status(http::StatusCode::BAD_REQUEST);

        cleanup_test_data(&pool, "TestGuest_VMP").await;
    }

    #[tokio::test]
    async fn rsvp_requires_guest_session() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_RRGS").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_RRGS", "password123").await;
        let server = test_server(pool.clone());

        // Get admin_pending session
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let session_cookie = response.cookie("session");

        // Try to access RSVP with admin_pending session (not a guest session)
        let response = server.get("/rsvp").add_cookie(session_cookie).await;
        // Returns 401 because RSVP requires guest session type
        response.assert_status(http::StatusCode::UNAUTHORIZED);

        cleanup_test_data(&pool, "TestAdmin_RRGS").await;
    }

    // ============================================================================
    // Admin Guest Management Tests
    // ============================================================================

    async fn get_admin_session(
        server: &TestServer,
        pool: &PgPool,
        test_name: &str,
    ) -> tower_cookies::Cookie<'static> {
        let admin_name = format!("Admin_{}", test_name);
        cleanup_test_data(pool, &admin_name).await;
        let (_, code) = create_test_admin(pool, &admin_name, "password123").await;

        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let pending_cookie = response.cookie("session");

        let response = server
            .post("/auth/admin/login")
            .add_cookie(pending_cookie)
            .json(&json!({"username": admin_name, "password": "password123"}))
            .await;

        response.cookie("session")
    }

    #[tokio::test]
    async fn admin_create_guest() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_CG").await;
        cleanup_test_data(&pool, "NewGuest_CG").await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "CG").await;

        let response = server
            .post("/admin/guests")
            .add_cookie(admin_cookie)
            .json(&json!({"name": "NewGuest_CG", "party_size": 3}))
            .await;

        response.assert_status(http::StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "NewGuest_CG");
        assert_eq!(body["party_size"], 3);
        assert!(body["invite_code"].as_str().unwrap().len() == 6);

        cleanup_test_data(&pool, "Admin_CG").await;
        cleanup_test_data(&pool, "NewGuest_CG").await;
    }

    #[tokio::test]
    async fn admin_list_guests() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_LG").await;
        cleanup_test_data(&pool, "TestGuest_LG").await;

        create_test_guest(&pool, "TestGuest_LG_1", 2).await;
        create_test_guest(&pool, "TestGuest_LG_2", 4).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "LG").await;

        let response = server.get("/admin/guests").add_cookie(admin_cookie).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(body["total"].as_i64().unwrap() >= 2);

        cleanup_test_data(&pool, "Admin_LG").await;
        cleanup_test_data(&pool, "TestGuest_LG").await;
    }

    #[tokio::test]
    async fn admin_update_guest() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_UG").await;
        cleanup_test_data(&pool, "TestGuest_UG").await;

        let (guest_id, _) = create_test_guest(&pool, "TestGuest_UG", 2).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "UG").await;

        let response = server
            .put(&format!("/admin/guests/{}", guest_id))
            .add_cookie(admin_cookie)
            .json(&json!({"name": "UpdatedGuest_UG", "party_size": 5}))
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "UpdatedGuest_UG");
        assert_eq!(body["party_size"], 5);

        cleanup_test_data(&pool, "Admin_UG").await;
        cleanup_test_data(&pool, "TestGuest_UG").await;
        cleanup_test_data(&pool, "UpdatedGuest_UG").await;
    }

    #[tokio::test]
    async fn admin_delete_guest() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_DG").await;
        cleanup_test_data(&pool, "TestGuest_DG").await;

        let (guest_id, _) = create_test_guest(&pool, "TestGuest_DG", 1).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "DG").await;

        let response = server
            .delete(&format!("/admin/guests/{}", guest_id))
            .add_cookie(admin_cookie)
            .await;

        response.assert_status(http::StatusCode::NO_CONTENT);

        // Verify guest is deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guests WHERE id = $1")
            .bind(guest_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);

        cleanup_test_data(&pool, "Admin_DG").await;
    }

    #[tokio::test]
    async fn admin_regenerate_code() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_RC").await;
        cleanup_test_data(&pool, "TestGuest_RC").await;

        let (guest_id, old_code) = create_test_guest(&pool, "TestGuest_RC", 1).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "RC").await;

        let response = server
            .post(&format!("/admin/guests/{}/code", guest_id))
            .add_cookie(admin_cookie)
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        let new_code = body["invite_code"].as_str().unwrap();
        assert_ne!(new_code, old_code);
        assert_eq!(new_code.len(), 6);

        cleanup_test_data(&pool, "Admin_RC").await;
        cleanup_test_data(&pool, "TestGuest_RC").await;
    }

    #[tokio::test]
    async fn admin_routes_require_auth() {
        let pool = test_pool().await;
        let server = test_server(pool);

        // All admin routes should return 401 without auth
        let response = server.get("/admin/guests").await;
        response.assert_status(http::StatusCode::UNAUTHORIZED);

        let response = server.get("/admin/dashboard").await;
        response.assert_status(http::StatusCode::UNAUTHORIZED);

        let response = server.get("/admin/events").await;
        response.assert_status(http::StatusCode::UNAUTHORIZED);
    }

    // ============================================================================
    // Admin Event Management Tests
    // ============================================================================

    #[tokio::test]
    async fn admin_create_event() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_CE").await;
        cleanup_test_data(&pool, "NewEvent_CE").await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "CE").await;

        let response = server
            .post("/admin/events")
            .add_cookie(admin_cookie)
            .json(&json!({
                "name": "NewEvent_CE",
                "event_type": "reception",
                "event_date": "2025-07-20",
                "event_time": "18:00",
                "location_name": "Grand Ballroom",
                "location_address": "456 Party Ave",
                "description": "A celebration!",
                "display_order": 1
            }))
            .await;

        response.assert_status(http::StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "NewEvent_CE");
        assert_eq!(body["event_type"], "reception");

        cleanup_test_data(&pool, "Admin_CE").await;
        cleanup_test_data(&pool, "NewEvent_CE").await;
    }

    #[tokio::test]
    async fn admin_update_event() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_UE").await;
        cleanup_test_data(&pool, "TestEvent_UE").await;

        let event_id = create_test_event(&pool, "TestEvent_UE", 1).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "UE").await;

        let response = server
            .put(&format!("/admin/events/{}", event_id))
            .add_cookie(admin_cookie)
            .json(&json!({
                "name": "UpdatedEvent_UE",
                "event_type": "brunch",
                "event_date": "2025-07-21",
                "event_time": "10:00",
                "location_name": "Breakfast Hall",
                "location_address": "789 Morning St",
                "description": null,
                "display_order": 2
            }))
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "UpdatedEvent_UE");
        assert_eq!(body["event_type"], "brunch");

        cleanup_test_data(&pool, "Admin_UE").await;
        cleanup_test_data(&pool, "TestEvent_UE").await;
        cleanup_test_data(&pool, "UpdatedEvent_UE").await;
    }

    #[tokio::test]
    async fn admin_delete_event() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_DE").await;
        cleanup_test_data(&pool, "TestEvent_DE").await;

        let event_id = create_test_event(&pool, "TestEvent_DE", 1).await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "DE").await;

        // Verify event exists via API
        let response = server
            .get("/admin/events")
            .add_cookie(admin_cookie.clone())
            .await;
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        let events = body["events"].as_array().unwrap();
        let found = events.iter().any(|e| e["id"] == event_id.to_string());
        assert!(found, "Event should be visible in events list");

        let response = server
            .delete(&format!("/admin/events/{}", event_id))
            .add_cookie(admin_cookie)
            .await;

        response.assert_status(http::StatusCode::NO_CONTENT);

        cleanup_test_data(&pool, "Admin_DE").await;
    }

    // ============================================================================
    // Dashboard Tests
    // ============================================================================

    #[tokio::test]
    async fn dashboard_returns_stats() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "Admin_DS").await;

        let server = test_server(pool.clone());
        let admin_cookie = get_admin_session(&server, &pool, "DS").await;

        let response = server
            .get("/admin/dashboard")
            .add_cookie(admin_cookie)
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();

        // Verify all expected fields exist
        assert!(body["total_guests"].is_number());
        assert!(body["total_expected_attendees"].is_number());
        assert!(body["rsvp_count"].is_number());
        assert!(body["pending_rsvps"].is_number());
        assert!(body["attending_count"].is_number());
        assert!(body["not_attending_count"].is_number());
        assert!(body["recent_rsvps"].is_array());

        cleanup_test_data(&pool, "Admin_DS").await;
    }

    // ============================================================================
    // Password Change Tests
    // ============================================================================

    #[tokio::test]
    async fn change_password_success() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_CP").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_CP", "oldpassword").await;
        let server = test_server(pool.clone());

        // Login
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let pending_cookie = response.cookie("session");

        let response = server
            .post("/auth/admin/login")
            .add_cookie(pending_cookie)
            .json(&json!({"username": "TestAdmin_CP", "password": "oldpassword"}))
            .await;
        let admin_cookie = response.cookie("session");

        // Change password
        let response = server
            .post("/admin/settings/password")
            .add_cookie(admin_cookie)
            .json(&json!({
                "current_password": "oldpassword",
                "new_password": "newpassword123"
            }))
            .await;

        response.assert_status_ok();

        cleanup_test_data(&pool, "TestAdmin_CP").await;
    }

    #[tokio::test]
    async fn change_password_wrong_current() {
        let pool = test_pool().await;
        cleanup_test_data(&pool, "TestAdmin_CPWC").await;

        let (_, code) = create_test_admin(&pool, "TestAdmin_CPWC", "correctpassword").await;
        let server = test_server(pool.clone());

        // Login
        let response = server.post("/auth/code").json(&json!({"code": code})).await;
        let pending_cookie = response.cookie("session");

        let response = server
            .post("/auth/admin/login")
            .add_cookie(pending_cookie)
            .json(&json!({"username": "TestAdmin_CPWC", "password": "correctpassword"}))
            .await;
        let admin_cookie = response.cookie("session");

        // Try to change with wrong current password
        let response = server
            .post("/admin/settings/password")
            .add_cookie(admin_cookie)
            .json(&json!({
                "current_password": "wrongpassword",
                "new_password": "newpassword123"
            }))
            .await;

        // Returns 400 Bad Request with "Current password is incorrect"
        response.assert_status(http::StatusCode::BAD_REQUEST);

        cleanup_test_data(&pool, "TestAdmin_CPWC").await;
    }
}
