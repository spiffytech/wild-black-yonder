use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
use spacedust::apis::contracts_api;
use spacedust::models::Contract;

pub async fn foo() -> String {
    let mut conf = Configuration::new();
    conf.bearer_access_token = Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiQURNSU4iLCJ2ZXJzaW9uIjoidjIuMS4yIiwicmVzZXRfZGF0ZSI6IjIwMjMtMTEtMTgiLCJpYXQiOjE3MDA1MzA3OTMsInN1YiI6ImFnZW50LXRva2VuIn0.UUsEfJX8ASMpb9Ag2EY0PN9GaH3w2HUvAhxYlKSf-cqX66P6r8MUFSGYvWyLpQiNSdtVLiiYfGEeSpU0isp6ekjL9FeYWYeEGKZxBlm5dX1G8hN8-O_DbSvq85kDHr8hlSUT04dS4dIKDMSkBbCu1x0PD1gp0JC4uGVBPpQMZnFFIaAjNXr17q3Zoqf0FVWqTIRwgC_fE0asyslGv_EfsGta6RBYkY2gE2i_y4xkaKd-3fP7CU-tI4x9N7A7-p3rCN5kZ3FCghBKoVhuCnEmPVv8A16kz21i-cPMTLtLJqe4XZL4tH3HEB8CUgirS1R9ahjSHHLeo_eWtQq0nL-66w".to_string());

    let agent = get_my_agent(&conf).await.expect("Idunno, a test error?");
    agent.data.symbol
}

pub async fn get_my_contracts(conf: Configuration) -> Vec<Contract> {
    contracts_api::get_contracts(&conf, None, None)
        .await
        .expect("Idunno, a test error?")
        .data
}
