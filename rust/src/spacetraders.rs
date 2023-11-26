use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
use spacedust::apis::{contracts_api, fleet_api, systems_api};
use spacedust::models::{Agent, Contract, PurchaseShipRequest, Ship, ShipType, Shipyard, Waypoint};

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

pub async fn system_waypoints(conf: &Configuration, cache: crate::WaypointsCache) -> Vec<Waypoint> {
    let agent = agent(conf).await;
    let system = agent
        .headquarters
        .split('-')
        .take(2)
        .collect::<Vec<&str>>()
        .join("-");
    println!("System: {}", system);

    if let Some(waypoints) = cache.get(&system).await {
        return waypoints;
    }

    let mut waypoints: Vec<Waypoint> = vec![];
    // API is 1-indexed
    let mut page = 1;
    loop {
        let response =
            systems_api::get_system_waypoints(conf, system.as_str(), Some(page), Some(20))
                .await
                .unwrap();
        waypoints.extend(response.data);
        println!("Meta: {:?}", response.meta);
        if response.meta.total <= waypoints.len() as i32 {
            break;
        }
        page += 1;
    }
    println!("Waypoints count: {:?}", waypoints.len());
    waypoints.sort_by_key(|w| w.r#type);

    cache.insert(system, waypoints.clone()).await;

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
    for ship in response.data.iter() {
        println!("Ship: {:#?}", ship);
    }
    response.data
}

pub async fn get_ship_nav_choices(waypoints: Vec<Waypoint>, ship: &Ship) -> Vec<(Waypoint, f64)> {
    let ship_location = &ship.nav.route.origin;

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
