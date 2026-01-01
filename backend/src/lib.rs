use std::sync::Arc;

use axum::{routing::get, Json, Router};
use http::header::{HeaderName, HeaderValue};
use serde::Serialize;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer};

pub mod config;

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

pub fn create_router() -> Router {
    create_router_with_rate_limit(true)
}

pub fn create_router_with_rate_limit(enable_rate_limit: bool) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
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
