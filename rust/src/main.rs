use maud::{html, Markup};
use spacedust::apis::configuration::Configuration;

use axum::middleware::{self, Next};
use axum::{http::Request, response::Response};
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

mod render;
mod spacetraders;

use render::page;

fn from_now(iso: String) -> String {
    let now = chrono::Utc::now();
    let deadline = chrono::DateTime::parse_from_rfc3339(&iso).unwrap();
    let duration = deadline.signed_duration_since(now);
    let duration = duration.to_std().unwrap();
    let duration = std::time::Duration::from_secs(duration.as_secs());
    let duration = humantime::format_duration(duration);
    duration.to_string()
}

fn contract_terms_html(terms: spacedust::models::ContractTerms) -> Markup {
    html! {
        dl class="[&>dt]:text-sm [&>dt]:font-semibold [&>dt]:italic" {
            dt {"Deadline"}
            dd {(from_now(terms.deadline))}

            dt {"Payment"}
            dd {
               div class="flex" {
                    "Accepted: " (terms.payment.on_accepted) ","
                    "Fulfilled: " (terms.payment.on_fulfilled)
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

fn contract_html(contract: spacedust::models::Contract) -> Markup {
    html! {
        dl class="[&>dt]:text-sm [&>dt]:font-semibold [&>dt]:italic" {
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

#[tokio::main]
async fn main() {
    let static_assets_service = ServeDir::new("public");

    let app = Router::new().route("/", get(|| async { spacetraders::foo().await }))

    .route(
        "/contracts",
        get(|| async {
            let mut conf = Configuration::new();
            conf.bearer_access_token = Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiQURNSU4iLCJ2ZXJzaW9uIjoidjIuMS4yIiwicmVzZXRfZGF0ZSI6IjIwMjMtMTEtMTgiLCJpYXQiOjE3MDA1MzA3OTMsInN1YiI6ImFnZW50LXRva2VuIn0.UUsEfJX8ASMpb9Ag2EY0PN9GaH3w2HUvAhxYlKSf-cqX66P6r8MUFSGYvWyLpQiNSdtVLiiYfGEeSpU0isp6ekjL9FeYWYeEGKZxBlm5dX1G8hN8-O_DbSvq85kDHr8hlSUT04dS4dIKDMSkBbCu1x0PD1gp0JC4uGVBPpQMZnFFIaAjNXr17q3Zoqf0FVWqTIRwgC_fE0asyslGv_EfsGta6RBYkY2gE2i_y4xkaKd-3fP7CU-tI4x9N7A7-p3rCN5kZ3FCghBKoVhuCnEmPVv8A16kz21i-cPMTLtLJqe4XZL4tH3HEB8CUgirS1R9ahjSHHLeo_eWtQq0nL-66w".to_string());
            let contracts = spacetraders::get_my_contracts(conf).await;
            page(
            html! {
                div {
                    header class="text-lg font-semibold" {"Contracts"}
                    @for contract in contracts {
                        (contract_html(contract))
                    }
                }
            }, None)
        }),
    )
    .fallback_service(static_assets_service)
    .layer(middleware::from_fn(caching_middleware));

    println!("Running!");
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
