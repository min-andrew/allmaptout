use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use http::{
    header::{HeaderName, HeaderValue},
    Method,
};
use serde::Serialize;
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

pub mod config;
pub mod error;
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

    if is_dev {
        CorsLayer::permissive()
    } else {
        // In production, restrict CORS to the frontend origin
        // Set CORS_ORIGIN to your production URL (e.g., https://example.com)
        let origin = std::env::var("CORS_ORIGIN").expect("CORS_ORIGIN must be set in production");

        CorsLayer::new()
            .allow_origin(origin.parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
    }
}

pub fn create_router() -> Router {
    create_router_with_rate_limit(true)
}

pub fn create_router_with_rate_limit(enable_rate_limit: bool) -> Router {
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

    #[tokio::test]
    async fn health_returns_ok() {
        // Set development mode for tests to avoid CORS_ORIGIN requirement
        std::env::set_var("RUST_ENV", "development");
        let server = TestServer::new(create_router_with_rate_limit(false)).unwrap();
        let response = server.get("/health").await;
        response.assert_status_ok();
    }
}
