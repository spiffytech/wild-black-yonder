use parking_lot::Mutex;
use std::sync::Arc;

use maud::{html, Markup};

use spacedust::apis::configuration::Configuration;
use spacedust::apis::fleet_api;
use spacedust::models::{NavigateShipRequest, ShipNavStatus, ShipType};

use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use serde::Deserialize;

use crate::render::page;
use crate::spacetraders;

use crate::fragments;

#[debug_handler]
pub async fn index(State(state): State<Arc<Mutex<Configuration>>>) -> Result<Markup, AppError> {
    let conf = &state.lock().clone();
    let agent = spacetraders::agent(conf).await;

    let contracts = spacetraders::get_my_contracts(conf).await;

    let waypoints = spacetraders::system_waypoints(conf).await;

    let ships = spacetraders::get_my_ships(conf).await;

    Ok(page(
        html! {
            div {
                header class="text-lg font-semibold" {"Contracts"}
                (fragments::agent_html(agent))
            }

            div {
                header class="text-lg font-semibold" {"My Ships"}
                (fragments::ships_html(ships))
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
    State(state): State<Arc<Mutex<Configuration>>>,
    Path(params): Path<ShipyardParams>,
) -> Result<Markup, AppError> {
    let conf = state.lock().clone();
    let shipyard = spacetraders::get_shipyard(&conf, &params.system, &params.waypoint).await;
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
    State(state): State<Arc<Mutex<Configuration>>>,
    Path(params): Path<ShipBuyParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = state.lock().clone();
    spacetraders::ship_buy(&conf, params.ship_type, params.waypoint).await;

    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize, Debug)]
pub struct ShipNavChoicesParams {
    ship_symbol: String,
}
#[debug_handler]
pub async fn ship_nav_choices(
    State(state): State<Arc<Mutex<Configuration>>>,
    Path(params): Path<ShipNavChoicesParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = state.lock().clone();
    let ship = *fleet_api::get_my_ship(&conf, params.ship_symbol.as_str())
        .await
        .unwrap()
        .data;
    let waypoints = spacetraders::system_waypoints(&conf).await;
    let waypoints = spacetraders::get_ship_nav_choices(waypoints, &ship).await;

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
    State(state): State<Arc<Mutex<Configuration>>>,
    Path(params): Path<ShipGoParams>,
) -> Result<impl IntoResponse, AppError> {
    let conf = state.lock().clone();
    let ship = fleet_api::get_my_ship(&conf, params.ship_symbol.as_str())
        .await
        .unwrap()
        .data;

    if ship.nav.status == ShipNavStatus::Docked {
        fleet_api::orbit_ship(&conf, ship.symbol.as_str())
            .await
            .unwrap();
    }

    fleet_api::navigate_ship(
        &conf,
        ship.symbol.as_str(),
        Some(NavigateShipRequest::new(params.waypoint)),
    )
    .await
    .unwrap();

    Ok(Redirect::to("/").into_response())
}

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
