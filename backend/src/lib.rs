use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
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
    use sqlx::postgres::PgPoolOptions;

    async fn test_pool() -> PgPool {
        dotenvy::dotenv().ok();
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn health_returns_ok() {
        std::env::set_var("RUST_ENV", "development");
        let pool = test_pool().await;
        let server = TestServer::new(create_router_with_rate_limit(pool, false)).unwrap();
        let response = server.get("/health").await;
        response.assert_status_ok();
    }
}
