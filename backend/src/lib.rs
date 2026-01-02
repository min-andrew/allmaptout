use std::sync::Arc;

use axum::{routing::get, Json, Router};
use http::{
    header::{HeaderName, HeaderValue},
    Method,
};
use serde::Serialize;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer};

pub mod config;
pub mod error;

pub use error::{AppError, Result};

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
    let is_dev = std::env::var("RUST_ENV").unwrap_or_default() == "development";
    create_router_with_rate_limit(!is_dev)
}

pub fn create_router_with_rate_limit(enable_rate_limit: bool) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer())
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ));

    if enable_rate_limit {
        let governor_config = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(10)
                .burst_size(20)
                .key_extractor(SmartIpKeyExtractor)
                .finish()
                .unwrap(),
        );
        router.layer(GovernorLayer {
            config: governor_config,
        })
    } else {
        router
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn health_returns_ok() {
        let server = TestServer::new(create_router_with_rate_limit(false)).unwrap();
        let response = server.get("/health").await;
        response.assert_status_ok();
    }
}
