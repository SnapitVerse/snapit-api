mod alchemy;
mod auth;
mod chain;
mod constants;
mod db;
mod error;
mod graph;
mod handlers;
mod openapi;
mod routes;

use std::sync::Arc;

use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};

#[tokio::main]
async fn main() {
    let config = constants::Constants::new();
    let config = Arc::new(config);

    let mongo_client = db::mongo::init_db(config.clone())
        .await
        .expect("Failed to initialize DB");
    let mongo_client = Arc::new(mongo_client);

    let provider = Provider::<Http>::try_from(config.chain_url.as_str()).unwrap();

    let wallet: LocalWallet = config
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(config.chain_id); // Set this to the chain ID of your network

    let ethers_client = Arc::new(SignerMiddleware::new(provider, wallet));

    let api_routes = routes::routes(config, mongo_client, ethers_client);

    // Start the server
    warp::serve(api_routes).run(([127, 0, 0, 1], 3030)).await;
}
