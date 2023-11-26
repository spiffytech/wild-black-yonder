use maud::{html, Markup};

use spacedust::apis::fleet_api;
use spacedust::models::{NavigateShipRequest, Ship, ShipNavStatus, ShipType};

use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use serde::Deserialize;

use crate::render::page;
use crate::spacetraders::{self, ShipOrShipSymbol, ShipWaypoint};

use crate::fragments;
use crate::AppStateShared;

pub struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[debug_handler]
pub async fn index(State(state): State<AppStateShared>) -> Result<Markup, AppError> {
    let conf = &state.conf;
    let agent = spacetraders::agent(conf).await;

    let contracts = spacetraders::get_my_contracts(conf).await;

    let system_symbol = spacetraders::agent_system(conf).await;
    let waypoints =
        spacetraders::system_waypoints(conf, system_symbol, state.waypoints_cache.clone()).await;

    let ships = spacetraders::get_my_ships(conf).await;
    let mut ships_with_waypoints: Vec<(Ship, ShipWaypoint)> = vec![];
    for ship in ships {
        let ship_waypoint = spacetraders::get_ship_with_waypoint(
            conf,
            ShipOrShipSymbol::Ship(ship),
            &state.waypoints_cache,
        )
        .await;
        ships_with_waypoints.push(ship_waypoint);
    }

    Ok(page(
        html! {
            div {
                header class="text-lg font-semibold" {"Contracts"}
                (fragments::agent_html(agent))
            }

            div {
                header class="text-lg font-semibold" {"My Ships"}
                (fragments::ships_html(ships_with_waypoints))
            }

            div {
                header class="text-lg font-semibold" {"Contracts"}
                @if contracts.is_empty() {
                    div {"You have no contracts."}
                }
                @for contract in contracts {
                    (fragments::contract_html(contract))
                }
            }

            div {
                header class="text-lg font-semibold" {"Waypoints"}
                (fragments::waypoints_html(waypoints))
            }
        },
        None,
    ))
}

#[derive(Deserialize, Debug)]
pub struct ShipyardParams {
    system: String,
    waypoint: String,
}
#[debug_handler]
pub async fn shipyard(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipyardParams>,
) -> Result<Markup, AppError> {
    let conf = &state.conf;
    let shipyard = spacetraders::get_shipyard(conf, &params.system, &params.waypoint).await;
    //println!("Shipyard: {:?}", shipyard);

    Ok(page(
        html! {
            div {"Shipyard " (shipyard.symbol.to_string())}
            (fragments::shipyard_html(shipyard))
        },
        None,
    ))
}

#[derive(Deserialize, Debug)]
pub struct ShipBuyParams {
    waypoint: String,
    ship_type: ShipType,
}
#[debug_handler]
pub async fn ship_buy(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipBuyParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    spacetraders::ship_buy(conf, params.ship_type, params.waypoint).await;

    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize, Debug)]
pub struct ShipParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    let symbol = ShipOrShipSymbol::Symbol(params.ship_symbol);
    let (ship, waypoint) =
        spacetraders::get_ship_with_waypoint(conf, symbol, &state.waypoints_cache).await;

    Ok(fragments::ship_html(ship, waypoint, None))
}

#[derive(Deserialize, Debug)]
pub struct ShipNavChoicesParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_nav_choices(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipNavChoicesParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    let ship = *fleet_api::get_my_ship(conf, params.ship_symbol.as_str())
        .await
        .unwrap()
        .data;
    let system_symbol = spacetraders::agent_system(conf).await;
    let waypoints =
        spacetraders::system_waypoints(conf, system_symbol, state.waypoints_cache.clone()).await;
    let waypoints = spacetraders::get_ship_nav_choices(&ship, waypoints).await;

    Ok(page(
        html! {
            @for (waypoint, dist) in waypoints {
                (fragments::waypoint_html(waypoint, Some((&ship, dist))))
            }
        },
        None,
    ))
}

#[derive(Deserialize, Debug)]
pub struct ShipGoParams {
    ship_symbol: String,
    waypoint: String,
}
#[debug_handler]
pub async fn ship_nav_go(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipGoParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    let ship = fleet_api::get_my_ship(conf, params.ship_symbol.as_str())
        .await
        .unwrap()
        .data;

    if ship.nav.status == ShipNavStatus::Docked {
        fleet_api::orbit_ship(conf, ship.symbol.as_str())
            .await
            .unwrap();
    }

    fleet_api::navigate_ship(
        conf,
        ship.symbol.as_str(),
        Some(NavigateShipRequest::new(params.waypoint)),
    )
    .await
    .unwrap();

    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize, Debug)]
pub struct ShipDockParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_dock(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipDockParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;

    fleet_api::dock_ship(conf, params.ship_symbol.as_str())
        .await
        .unwrap();

    let (ship, waypoint) = spacetraders::get_ship_with_waypoint(
        conf,
        ShipOrShipSymbol::Symbol(params.ship_symbol),
        &state.waypoints_cache,
    )
    .await;

    Ok(fragments::ship_html(ship, waypoint, None))
}

#[derive(Deserialize, Debug)]
pub struct ShipOrbitParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_orbit(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipOrbitParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;

    fleet_api::orbit_ship(conf, params.ship_symbol.as_str())
        .await
        .unwrap();

    let (ship, ship_waypoint) = spacetraders::get_ship_with_waypoint(
        conf,
        ShipOrShipSymbol::Symbol(params.ship_symbol),
        &state.waypoints_cache,
    )
    .await;

    Ok(fragments::ship_html(ship, ship_waypoint, None))
}

#[derive(Deserialize, Debug)]
pub struct ShipRefuelParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_refuel(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipRefuelParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    let ship = spacetraders::ship_refuel(conf, ShipOrShipSymbol::Symbol(params.ship_symbol)).await;
    let (ship, waypoint) = spacetraders::get_ship_with_waypoint(
        conf,
        ShipOrShipSymbol::Ship(ship),
        &state.waypoints_cache,
    )
    .await;

    Ok(fragments::ship_html(ship, waypoint, None))
}

#[derive(Deserialize, Debug)]
pub struct ShipExtractParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_extract(
    State(state): State<AppStateShared>,
    Path(params): Path<ShipExtractParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = &state.conf;
    let symbol = ShipOrShipSymbol::Symbol(params.ship_symbol);
    let r#yield = spacetraders::ship_extract(conf, symbol.clone()).await;

    let (ship, waypoint) =
        spacetraders::get_ship_with_waypoint(conf, symbol, &state.waypoints_cache).await;

    Ok(fragments::ship_html(ship, waypoint, Some(r#yield)))
}
