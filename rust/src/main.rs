use moka::future::Cache;
//use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use spacedust::apis::configuration::Configuration;
use spacedust::models::Waypoint;

use axum::middleware::{self, Next};
use axum::{http::Request, response::Response};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

mod fragments;
mod render;
mod routes;
mod spacetraders;

/**
 * tower-http's ServeDir doesn't let us control caching for static files, and
 * the browser's default behavior is to just cache forever. So stupid.
 */
async fn caching_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    /*
    headers.insert(
        "cache-control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("expires", "0".parse().unwrap());
    */
    // This stupidly means caching is allowed, as long as it's always
    // revalidated.
    headers.insert("cache-control", "no-cache".parse().unwrap());

    response
}

pub type WaypointsCache = Cache<String, Vec<Waypoint>>;
pub struct AppState {
    conf: Configuration,
    waypoints_cache: WaypointsCache,
}

pub type AppStateShared = Arc<AppState>;

#[tokio::main]
async fn main() {
    let static_assets_service = ServeDir::new("public");

    let conf = {
        let mut conf = Configuration::new();
        conf.bearer_access_token = Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiQURNSU4iLCJ2ZXJzaW9uIjoidjIuMS4yIiwicmVzZXRfZGF0ZSI6IjIwMjMtMTEtMTgiLCJpYXQiOjE3MDA1MzA3OTMsInN1YiI6ImFnZW50LXRva2VuIn0.UUsEfJX8ASMpb9Ag2EY0PN9GaH3w2HUvAhxYlKSf-cqX66P6r8MUFSGYvWyLpQiNSdtVLiiYfGEeSpU0isp6ekjL9FeYWYeEGKZxBlm5dX1G8hN8-O_DbSvq85kDHr8hlSUT04dS4dIKDMSkBbCu1x0PD1gp0JC4uGVBPpQMZnFFIaAjNXr17q3Zoqf0FVWqTIRwgC_fE0asyslGv_EfsGta6RBYkY2gE2i_y4xkaKd-3fP7CU-tI4x9N7A7-p3rCN5kZ3FCghBKoVhuCnEmPVv8A16kz21i-cPMTLtLJqe4XZL4tH3HEB8CUgirS1R9ahjSHHLeo_eWtQq0nL-66w".to_string());
        conf
    };

    let waypoints_cache: WaypointsCache = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 5))
        .build();

    let app_state = Arc::new(AppState {
        conf,
        waypoints_cache,
    });

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/shipyard/:system/:waypoint", get(routes::shipyard))
        .route(
            "/waypoints/:waypoint/buy_ship/:ship_type",
            post(routes::ship_buy),
        )
        .route(
            "/ship_nav/:ship_symbol/choices",
            get(routes::ship_nav_choices),
        )
        .route(
            "/ship_nav/:ship_symbol/go/:waypoint",
            post(routes::ship_nav_go),
        )
        .route("/ship_nav/:ship_symbol/dock", post(routes::ship_dock))
        .route("/ship_nav/:ship_symbol/orbit", post(routes::ship_orbit))
        .route("/ship_nav/:ship_symbol/refuel", post(routes::ship_refuel))
        .with_state(app_state)
        .fallback_service(static_assets_service)
        .layer(middleware::from_fn(caching_middleware));

    println!("Running!");
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
