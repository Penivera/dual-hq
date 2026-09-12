use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use tower_governor::{
    errors::GovernorError,
    governor::GovernorConfigBuilder,
    key_extractor::KeyExtractor,
    GovernorLayer,
};

use crate::{db::AppState, handlers::auth};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientIpKeyExtractor;

impl KeyExtractor for ClientIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let headers = req.headers();
        if let Some(val) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(ip_str) = val.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse() {
                    return Ok(ip);
                }
            }
        }
        if let Some(val) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            if let Ok(ip) = val.trim().parse() {
                return Ok(ip);
            }
        }
        if let Some(info) = req.extensions().get::<axum::extract::ConnectInfo<SocketAddr>>() {
            return Ok(info.0.ip());
        }
        // Fallback for tests/local requests without ConnectInfo
        Ok(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
    }
}

fn create_rate_limiter(
    per_millis: u64,
    burst_size: u32,
) -> GovernorLayer<ClientIpKeyExtractor, ::governor::middleware::NoOpMiddleware, Body> {
    let mut builder = GovernorConfigBuilder::default();
    builder.per_millisecond(per_millis);
    builder.burst_size(burst_size);
    let config = builder.key_extractor(ClientIpKeyExtractor).finish().unwrap();

    GovernorLayer::new(Arc::new(config)).error_handler(|_| {
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"detail":"too many requests, slow down"}"#))
            .unwrap()
    })
}

pub fn router() -> Router<AppState> {
    // 10 req/min => replenish every 6000ms, burst 10
    let login_limiter = create_rate_limiter(6000, 10);
    let register_limiter = create_rate_limiter(6000, 10);
    // 20 req/min => replenish every 3000ms, burst 20
    let refresh_limiter = create_rate_limiter(3000, 20);

    let login_route = Router::new()
        .route("/login", post(auth::login))
        .layer(login_limiter);

    let register_route = Router::new()
        .route("/register", post(auth::register))
        .layer(register_limiter);

    let refresh_route = Router::new()
        .route("/refresh", post(auth::refresh_token))
        .layer(refresh_limiter);

    Router::new()
        .merge(login_route)
        .merge(register_route)
        .merge(refresh_route)
        .route("/logout", post(auth::logout))
        .route("/verify", get(auth::verify_email).post(auth::verify_email))
        .route("/resend-verification", post(auth::resend_verification))
}
