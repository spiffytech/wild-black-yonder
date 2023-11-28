use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
use spacedust::apis::{contracts_api, fleet_api, systems_api};
use spacedust::models::{
    Agent, Contract, ExtractResourcesRequest, ExtractionYield, Market, PurchaseShipRequest,
    RefuelShipRequest, SellCargoRequest, Ship, ShipType, Shipyard, TradeSymbol, Waypoint,
    WaypointTraitSymbol,
};

use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone)]
pub enum ShipOrShipSymbol {
    Ship(Ship),
    Symbol(String),
}

impl ShipOrShipSymbol {
    pub async fn get(&self, conf: &Configuration) -> Ship {
        match self {
            ShipOrShipSymbol::Ship(ship) => ship.clone(),
            ShipOrShipSymbol::Symbol(ship_symbol) => get_ship(conf, ship_symbol.as_str()).await,
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            ShipOrShipSymbol::Ship(ship) => ship.symbol.clone(),
            ShipOrShipSymbol::Symbol(ship_symbol) => ship_symbol.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WaypointFeatures {
    Marketplace,
    Shipyard,
    Fuel,
}

pub struct ShipWaypoint {
    pub waypoint: Waypoint,
    pub features: Vec<WaypointFeatures>,
    pub market: Option<Market>,
}

pub async fn agent(conf: &Configuration) -> Agent {
    let agent = get_my_agent(conf).await.expect("Idunno, a test error?");
    *agent.data
}

pub async fn get_my_contracts(conf: &Configuration) -> Vec<Contract> {
    contracts_api::get_contracts(conf, None, None)
        .await
        .expect("Idunno, a test error?")
        .data
}

pub async fn agent_system(conf: &Configuration) -> String {
    let agent = agent(conf).await;
    agent
        .headquarters
        .split('-')
        .take(2)
        .collect::<Vec<&str>>()
        .join("-")
}

pub async fn system_waypoints(
    conf: &Configuration,
    system_symbol: String,
    waypoints_cache: crate::WaypointsCache,
) -> Vec<Waypoint> {
    let waypoints = waypoints_cache
        .try_get_with::<_, ()>(system_symbol.clone(), async {
            let mut waypoints: Vec<Waypoint> = vec![];
            // API is 1-indexed
            let mut page = 1;
            loop {
                let response = systems_api::get_system_waypoints(
                    conf,
                    system_symbol.as_str(),
                    Some(page),
                    Some(20),
                )
                .await
                .unwrap();
                waypoints.extend(response.data);
                if response.meta.total <= waypoints.len() as i32 {
                    break;
                }
                page += 1;
            }
            waypoints.sort_by_key(|w| w.r#type);
            Ok(waypoints)
        })
        .await
        .unwrap();

    // Not strictly necessary, but cache metadata is kinda eventually consistent
    // unless we call this, and it goofs up my ability to debug stuff if I can't
    // trust e.g., how many entries are in the cache.
    waypoints_cache.run_pending_tasks().await;

    waypoints
}

pub async fn get_shipyard(
    conf: &Configuration,
    system_symbol: &str,
    waypoint_symbol: &str,
) -> Shipyard {
    let response = systems_api::get_shipyard(conf, system_symbol, waypoint_symbol)
        .await
        .unwrap();
    *response.data
}

pub async fn ship_buy(conf: &Configuration, ship_type: ShipType, waypoint: String) {
    fleet_api::purchase_ship(conf, Some(PurchaseShipRequest::new(ship_type, waypoint)))
        .await
        .unwrap();
}

pub async fn get_my_ships(conf: &Configuration) -> Vec<Ship> {
    let response = fleet_api::get_my_ships(conf, None, None).await.unwrap();
    response.data
}

pub async fn get_ship_nav_choices(ship: &Ship, waypoints: Vec<Waypoint>) -> Vec<(Waypoint, f64)> {
    let ship_waypoint = &ship.nav.waypoint_symbol;
    let ship_location = waypoints
        .iter()
        .find(|w| w.symbol == *ship_waypoint)
        .expect("Ship not at waypoint")
        .clone();

    let mut distances = waypoints
        .into_iter()
        .map(|w| {
            let dist =
                (((w.x - ship_location.x).pow(2) + (w.y - ship_location.y).pow(2)) as f64).sqrt();
            (w, dist)
        })
        .collect::<Vec<(Waypoint, f64)>>();

    distances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    distances
}

pub async fn get_ship(conf: &Configuration, ship_symbol: &str) -> Ship {
    *fleet_api::get_my_ship(conf, ship_symbol)
        .await
        .unwrap()
        .data
}

pub async fn get_ship_with_waypoint(
    conf: &Configuration,
    ship: ShipOrShipSymbol,
    waypoints_cache: &crate::WaypointsCache,
) -> (Ship, ShipWaypoint) {
    let ship = ship.get(conf).await;

    println!(
        "Waypoint symbol: {}, cache entry count: {}",
        ship.nav.system_symbol,
        waypoints_cache.entry_count()
    );
    let waypoints = system_waypoints(
        conf,
        ship.nav.system_symbol.clone(),
        waypoints_cache.clone(),
    )
    .await;
    let waypoint = waypoints
        .iter()
        .find(|w| w.symbol == ship.nav.waypoint_symbol)
        .unwrap()
        .clone();

    let mut waypoint_features: Vec<WaypointFeatures> = vec![];
    waypoint
        .traits
        .iter()
        .for_each(|trait_| match trait_.symbol {
            WaypointTraitSymbol::Marketplace => {
                waypoint_features.push(WaypointFeatures::Marketplace);
            }
            WaypointTraitSymbol::Shipyard => {
                waypoint_features.push(WaypointFeatures::Shipyard);
            }
            _ => {}
        });

    let mut market: Option<Market> = None;
    if waypoint_features.contains(&WaypointFeatures::Marketplace) {
        let market_ = *systems_api::get_market(conf, &waypoint.system_symbol, &waypoint.symbol)
            .await
            .unwrap()
            .data;

        if market_
            .exchange
            .iter()
            .any(|e| e.symbol == TradeSymbol::Fuel)
        {
            waypoint_features.push(WaypointFeatures::Fuel);
        }

        market = Some(market_);
    }

    (
        ship,
        ShipWaypoint {
            waypoint,
            features: waypoint_features,
            market,
        },
    )
}

pub async fn ship_refuel(conf: &Configuration, symbol: ShipOrShipSymbol) -> Ship {
    fleet_api::refuel_ship(conf, &symbol.symbol(), Some(RefuelShipRequest::new()))
        .await
        .unwrap();

    symbol.get(conf).await
}

pub async fn ship_extract(conf: &Configuration, ship: ShipOrShipSymbol) -> ExtractionYield {
    let result =
        *fleet_api::extract_resources(conf, &ship.symbol(), Some(ExtractResourcesRequest::new()))
            .await
            .unwrap()
            .data;

    *result.extraction.r#yield
}

pub async fn ship_cargo_dump(conf: &Configuration, ship: ShipOrShipSymbol) {
    let ship = ship.get(conf).await;

    // Marketplaces only buy certain goods, and API calls will fail if we try to
    // sell anything else
    let marketplace =
        *systems_api::get_market(conf, &ship.nav.system_symbol, &ship.nav.waypoint_symbol)
            .await
            .unwrap()
            .data;
    let goods_sellable = marketplace
        .imports
        .iter()
        .map(|i| i.symbol)
        .collect::<Vec<TradeSymbol>>();

    let inventory = ship.cargo.inventory;

    for item in inventory {
        println!("Checking if {:?} is sellable", item.symbol);
        if !goods_sellable.contains(&item.symbol) {
            continue;
        }
        println!("Selling {} units of {:?}", item.units, item.symbol);

        fleet_api::sell_cargo(
            conf,
            &ship.symbol,
            Some(SellCargoRequest::new(item.symbol, item.units)),
        )
        .await
        .unwrap();
    }
}

pub fn map_data(waypoints: Vec<Waypoint>, ships: Vec<Ship>) -> String {
    let mut waypoint_nodes = waypoints
        .iter()
        .map(|waypoint| {
            json!({
                "data": {
                    "id": waypoint.symbol.clone(),
                },
                "position": {
                    "x": waypoint.x,
                    "y": waypoint.y,
                },
                "classes": ["waypoint", waypoint.r#type.to_string().to_lowercase()]
            })
        })
        .collect::<Vec<_>>();

    let mut waypoint_edges = waypoints
        .iter()
        .flat_map(|waypoint| {
            waypoint.orbitals.iter().map(|orbital| {
                json!({
                    "data": {
                        "id": format!("{}-{}", waypoint.symbol, orbital.symbol),
                        "source": waypoint.symbol,
                        "target": orbital.symbol,
                    },
                    "classes": ["orbital"]
                })
            })
        })
        .collect::<Vec<_>>();

    let mut ship_nodes: Vec<JsonValue> = vec![];
    let mut ship_edges: Vec<JsonValue> = vec![];
    ships.iter().for_each(|ship| {
        let ship_waypoint_symbol = &ship.nav.waypoint_symbol;
        let ship_waypoint = waypoints
            .iter()
            .find(|w| w.symbol == *ship_waypoint_symbol)
            .expect("Ship not at waypoint")
            .clone();

        ship_nodes.push(json!({
            "data": {
                "id": ship.symbol.clone(),
            },
            "position": {
                "x": ship_waypoint.x,
                "y": ship_waypoint.y,
            },
            "classes": ["ship"]
        }));
        ship_edges.push(json!({
            "data": {
                "id": format!("{}-{}", ship.symbol, ship_waypoint_symbol),
                "source": ship.symbol,
                "target": ship_waypoint_symbol,
            },
            "classes": ["orbital"]
        }));
    });

    waypoint_nodes.extend(ship_nodes);
    let map_nodes = waypoint_nodes;
    waypoint_edges.extend(ship_edges);
    let map_edges = waypoint_edges;

    serde_json::to_string(&json!({
        "nodes": &map_nodes,
        "edges": &map_edges
    }))
    .unwrap()
}
