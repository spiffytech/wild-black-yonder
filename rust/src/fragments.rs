use maud::{html, Markup};
use spacedust::models::{ExtractionYield, Ship, ShipNavStatus, Shipyard, WaypointTraitSymbol};

use crate::spacetraders::{ShipWaypoint, WaypointFeatures};

fn from_now(iso: String) -> String {
    let now = chrono::Utc::now();
    let deadline = chrono::DateTime::parse_from_rfc3339(&iso).unwrap();
    let duration = deadline.signed_duration_since(now);
    let duration = duration.to_std().unwrap();
    let duration = std::time::Duration::from_secs(duration.as_secs());
    let duration = humantime::format_duration(duration);
    duration.to_string()
}

pub fn agent_html(agent: spacedust::models::Agent) -> Markup {
    html! {
        dl class="[&_dt]:text-sm [&_dt]:font-semibold [&_dt]:italic" {
            dt {"Symbol"}
            dd {(agent.symbol)}

            dt {"Faction"}
            dd {(agent.starting_faction)}

            dt {"Credits"}
            dd {(agent.credits)}

            dt {"HQ"}
            dd {(agent.headquarters)}
        }
    }
}

pub fn contract_terms_html(terms: spacedust::models::ContractTerms) -> Markup {
    html! {
        dl class="" {
            dt {"Deadline"}
            dd {(from_now(terms.deadline))}

            dt {"Payment"}
            dd {
               div class="flex items-baseline" {
                    span class="text-sm" {"Accepted: "} (terms.payment.on_accepted) ", "
                    span class="text-sm" {"Fulfilled: "} (terms.payment.on_fulfilled)
                }
            }

            @if let Some(delivers) = terms.deliver {
                @for deliver in &delivers {
                    dt {"Deliver"}
                    dd {
                        (deliver.units_fulfilled)"/"(deliver.units_required) " " (deliver.trade_symbol) " to " (deliver.destination_symbol)
                    }
                }
            }
        }
    }
}

pub fn contract_html(contract: spacedust::models::Contract) -> Markup {
    html! {
        dl class="[&_dt]:text-sm [&_dt]:font-semibold [&_dt]:italic" {
            dt {"ID"}
            dd {(contract.id)}

            dt {"Faction"}
            dd {(contract.faction_symbol)}

            dt {"Accepted"}
            dd {(contract.accepted)}

            dt {"Fulfilled"}
            dd {(contract.fulfilled)}

            dt {"Expiration"}
            dd {(from_now(contract.expiration))}

            dt {"Terms"}
            dd class="ml-4" {(contract_terms_html(*contract.terms))}
        }
    }
}

pub fn waypoint_html(
    waypoint: spacedust::models::Waypoint,
    nav_distance: Option<(&Ship, f64)>,
) -> Markup {
    html! {
        div id=(waypoint.symbol) {
            div class="capitalize flex gap-2 items-baseline" {
                span class="text-lg font-semibold" {(waypoint.r#type.to_string().to_lowercase())}
                span class="text-sm text-gray-700 italic" {
                    (waypoint.symbol)
                    @if let Some((ship, dist)) = nav_distance {
                        form method="POST" action=(format!("/ship_nav/{ship_symbol}/go/{waypoint}", ship_symbol=ship.symbol, waypoint=waypoint.symbol)) up-layer="parent" {
                            button {
                                (dist) i class="bi-arrow-right" {}
                            }
                        }
                    }
                }
            }
            ul class="flex gap-2" {
                @for waypoint_trait in &waypoint.traits {
                    li title=(waypoint_trait.description) {
                        @match waypoint_trait.symbol {
                            WaypointTraitSymbol::Shipyard => {
                                a
                                    href={"/shipyard/" (waypoint.system_symbol) "/" (waypoint.symbol)}
                                    class="underline"
                                    up-layer="new"
                                    up-history="false"
                                {(waypoint_trait.name)}
                            }
                            _ => {
                                (waypoint_trait.name)
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn waypoints_html(waypoints: Vec<spacedust::models::Waypoint>) -> Markup {
    let by_type = waypoints.clone();
    let (asteroids, by_type): (Vec<_>, Vec<_>) = by_type
        .into_iter()
        .partition(|w| w.r#type == spacedust::models::WaypointType::Asteroid);

    let (fuel_stations, by_type): (Vec<_>, Vec<_>) = by_type
        .into_iter()
        .partition(|w| w.r#type == spacedust::models::WaypointType::FuelStation);

    let by_feature = waypoints;
    let (shipyards, _by_feature): (Vec<_>, Vec<_>) = by_feature.into_iter().partition(|w| {
        w.traits
            .iter()
            .any(|t| t.symbol == WaypointTraitSymbol::Shipyard)
    });

    html! {
        div {
            header {"Features"}

            detail {
                summary class="text-lg font-semibold" {"Shipyards"}
                ul class="ml-4" {
                    @for waypoint in shipyards {
                        li {(waypoint_html(waypoint, None))}
                    }
                }
            }
        }

        @for waypoint in by_type {
            (waypoint_html(waypoint, None))
        }
        @for waypoint in fuel_stations {
            (waypoint_html(waypoint, None))
        }
        @for waypoint in asteroids {
            (waypoint_html(waypoint, None))
        }
    }
}

pub fn shipyard_html(shipyard: Shipyard) -> Markup {
    let Some(ships) = shipyard.ships else {
        return html! {
            div {"No ships available."}
        };
    };

    for ship in ships.iter() {
        println!("Ship: {:#?}", ship);
    }

    html! {
        ul {
            @for ship in ships {
                li {
                    div title=(ship.description) {(ship.name)}
                    div {"Supply: " (ship.supply.to_string().to_lowercase())}
                    div {"Price: " (ship.purchase_price)}
                    div {"Fuel: " (ship.frame.fuel_capacity)}
                    form
                        method="POST"
                        action={"/waypoints/" (shipyard.symbol) "/buy_ship/" (ship.r#type.unwrap().to_string())}
                        up-layer="parent"
                    {
                        button type="submit" class="border rounded-md bg-gray-100 px-2 py-1" {"Buy"}
                    }
                }
            }

        }
    }
}

pub fn ship_html(
    ship: Ship,
    ship_waypoint: ShipWaypoint,
    extraction_yield: Option<ExtractionYield>,
) -> Markup {
    let on_cooldown = ship.cooldown.expiration;
    let in_transit = ship.nav.status == ShipNavStatus::InTransit;

    let poll = if on_cooldown.is_some() || in_transit {
        Some(())
    } else {
        None
    };

    html! {
        // The special class is necessary because unpoly needs some way to
        // automatically target the element, otherwise the polling fails.
        li
            class={"ship ship-" (ship.symbol)}
            up-poll[poll.is_some()]
            up-interval=[poll.map(|_| "5000")]
            up-source=[poll.map(|_| format!("/ship/{}", ship.symbol))]
        {
            div {
                (format!(
                    "{} {} (Fuel {}/{}) {:?}",
                    ship.symbol,
                    ship.registration.role.to_string(),
                    ship.fuel.current,
                    ship.fuel.capacity,
                    ship.nav.status
                ))

                @if let Some(cooldown_expiration) = on_cooldown.clone() {
                    (format!(" Cooldown: {}", from_now(cooldown_expiration.clone())))
                }

                @match ship.nav.status {
                    ShipNavStatus::InTransit => {
                        (format!(" ETA: {}", from_now(ship.nav.route.arrival)))
                    },
                    ShipNavStatus::InOrbit => {
                        button up-href={"/ship_nav/" (ship.symbol) "/dock"} up-method="post" up-target=".ship" {
                            i class="bi-arrow-bar-down" {}
                        }
                    },

                    ShipNavStatus::Docked => {
                        button up-href={"/ship_nav/" (ship.symbol) "/orbit"} up-method="post" up-target=".ship" {
                            i class="bi-arrow-bar-up" {}
                        }
                    },
                }

                a
                    href=(format!("/ship_nav/{ship_symbol}/choices", ship_symbol=ship.symbol))
                    up-layer="new"
                    up-history="false"
                {
                    i class="bi-airplane ml-2" {}
                }

                button
                    disabled[on_cooldown.is_some()]
                    class=(on_cooldown.clone().map_or("", |_| "text-gray-400"))
                    up-href={"/ship_nav/" (ship.symbol) "/extract"}
                    up-method="post"
                    up-target=".ship"
                    {
                    i class="bi-minecart-loaded" {}
                }
            }

            div class="flex gap-x-2" {
                a
                    href={"#" (ship.nav.waypoint_symbol)}
                    class="underline decoration-dotted"

                {(ship.nav.waypoint_symbol)}

                @for feature in ship_waypoint.features.iter() {
                    @match feature {
                        WaypointFeatures::Marketplace => {
                            i title="Marketplace" class="bi-shop" {}
                        },
                        WaypointFeatures::Shipyard => {
                            i title="Shipyard" class="bi-rocket" {}
                        },
                        WaypointFeatures::Fuel => {
                            @let ship_fuel_full = ship.fuel.current == ship.fuel.capacity;
                            button up-href={"ship_nav/" (ship.symbol) "/refuel"} up-method="post" up-target=".ship" {
                                i title="Fuel" class={"bi-fuel-pump " (if ship_fuel_full {"text-gray-400"} else {""})}  {}
                            }
                        },
                    }
                }
            }

            @if ship.cargo.units > 0 {
                details open {
                    summary {"Cargo (" (ship.cargo.units) "/" (ship.cargo.capacity) ")"}
                    ul class="flex gap-x-2" {
                        @for cargo_item in ship.cargo.inventory {
                            li {(cargo_item.name) " " (cargo_item.units)}
                        }
                    }

                    @if ship_waypoint.features.contains(&WaypointFeatures::Marketplace) {
                        button
                            up-href={"/ship_cargo/" (ship.symbol) "/dump"}
                            up-method="post"
                            up-target=".ship"
                            class="flex gap-x-1 items-baseline"
                        {
                            i class="bi-upload" {}
                            "Dump cargo"
                        }
                    }
                }
            }

            @if let Some(r#yield) = extraction_yield {
                div {
                    (format!("Extracted {} {}", r#yield.units, r#yield.symbol.to_string()))
                }
            }
        }
    }
}

pub fn ships_html(ships: Vec<(Ship, ShipWaypoint)>) -> Markup {
    html! {
        ul class="ships [&>li]:mb-2" {
            @for (ship, waypoint) in ships {
                (ship_html(ship, waypoint, None))
            }
        }
    }
}
