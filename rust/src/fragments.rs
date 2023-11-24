use maud::{html, Markup};
use spacedust::models::{Ship, Shipyard, WaypointTraitSymbol};

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

pub fn waypoint_html(waypoint: spacedust::models::Waypoint) -> Markup {
    html! {
        div {
            div class="capitalize flex gap-2 items-baseline" {
                span class="text-lg font-semibold" {(waypoint.r#type.to_string().to_lowercase())}
                span class="text-sm text-gray-700 italic" {
                    (waypoint.symbol)
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
    let (shipyards, by_feature): (Vec<_>, Vec<_>) = by_feature.into_iter().partition(|w| {
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
                        li {(waypoint_html(waypoint))}
                    }
                }
            }
        }

        @for waypoint in by_type {
            (waypoint_html(waypoint))
        }
        @for waypoint in fuel_stations {
            (waypoint_html(waypoint))
        }
        @for waypoint in asteroids {
            (waypoint_html(waypoint))
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
                        action={"/ships/" (shipyard.symbol) "/buy/" (ship.r#type.unwrap().to_string())}
                        up-layer="parent"
                    {
                        button type="submit" class="border rounded-md bg-gray-100 px-2 py-1" {"Buy"}
                    }
                }
            }

        }
    }
}

pub fn ships_html(ships: Vec<Ship>) -> Markup {
    html! {
        ul {
            @for ship in ships {
                li {
                    (ship.registration.name) " " (ship.registration.role.to_string()) " " (ship.symbol) "(Fuel " (ship.fuel.current) "/" (ship.fuel.capacity) ")"
                }
            }
        }
    }
}
