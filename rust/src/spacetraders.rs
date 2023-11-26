use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
use spacedust::apis::{contracts_api, fleet_api, systems_api};
use spacedust::models::{
    Agent, Contract, PurchaseShipRequest, Ship, ShipType, Shipyard, Waypoint, WaypointTraitSymbol,
};

pub enum WaypointFeatures {
    Marketplace,
    Shipyard,
    Fuel,
}

pub struct ShipWaypoint {
    pub waypoint: Waypoint,
    pub features: Vec<WaypointFeatures>,
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
    ship_symbol: &str,
    waypoints_cache: &crate::WaypointsCache,
) -> (Ship, ShipWaypoint) {
    let ship = get_ship(conf, ship_symbol).await;

    println!(
        "Waypoint symbol: {}, cache entry count: {}",
        ship.nav.system_symbol,
        waypoints_cache.entry_count()
    );
    let waypoints = waypoints_cache.get(&ship.nav.system_symbol).await.unwrap();
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

    (
        ship,
        ShipWaypoint {
            waypoint,
            features: waypoint_features,
        },
    )
}
