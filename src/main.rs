mod chain;
mod constants;
mod db;
mod graph;
mod handlers;

use std::sync::Arc;

use constants::Constants;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::H256;
use handlers::get_auction::{get_auction, GetAuctionQueryParams};
use handlers::get_nft::{get_nft, GetNftQueryParams};
use handlers::get_owner_tokens::{get_owner_tokens_handler, GetOwnerTokensQueryParams};
use handlers::mint_nft::mint_nft_handler;
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = constants::Constants::new();
    let config = Arc::new(config);

    let mongo_client = db::mongo::init_db().await.expect("Failed to initialize DB");
    let mongo_client = Arc::new(mongo_client);

    let provider = Provider::<Http>::try_from(config.chain_url.as_str()).unwrap();

    let wallet: LocalWallet = config
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(config.chain_id); // Set this to the chain ID of your network

    let ethers_client = Arc::new(SignerMiddleware::new(provider, wallet));

    let config_filter = with_config(config.clone());
    let mongo_client_filter = with_mongo_client(mongo_client);
    let ethers_client_filter = with_ethers_client(ethers_client);

    // GET endpoint at /
    let get_route = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::json(&"Welcome to the Rust API!"));

    // POST endpoint at /echo
    let post_route = warp::post()
        .and(warp::path("echo"))
        .and(warp::body::json())
        .map(|body: EchoRequest| {
            warp::reply::json(&EchoResponse {
                message: body.message,
            })
        });

    let mint_nft_route = warp::post()
        .and(warp::path("api"))
        .and(warp::path("mint"))
        .and(warp::body::json())
        .and(mongo_client_filter.clone())
        .and(config_filter.clone())
        .and_then(mint_nft_handler);

    let get_nft_route = warp::get()
        .and(mongo_client_filter.clone())
        .and(warp::path("api"))
        .and(warp::path("token"))
        .and(warp::path::param::<String>()) // Capture {id}.json as a String
        .and(warp::query::<GetNftQueryParams>()) // Use query to capture with_owner
        .and(config_filter.clone())
        .and_then(get_nft);

    let get_owner_tokens_route = warp::get()
        .and(mongo_client_filter.clone())
        .and(warp::path("api"))
        .and(warp::path("get-owner-tokens"))
        .and(warp::query::<GetOwnerTokensQueryParams>())
        .and(config_filter.clone())
        .and_then(get_owner_tokens_handler);

    let get_auction_route = warp::get()
        .and(warp::path("api"))
        .and(warp::path("auction"))
        .and(ethers_client_filter.clone())
        .and(warp::query::<GetAuctionQueryParams>()) // Use query to capture with_owner
        .and(config_filter.clone())
        .and_then(get_auction);

    // Combine the routes
    let routes = get_route
        .or(post_route)
        .or(mint_nft_route)
        .or(get_owner_tokens_route)
        .or(get_nft_route)
        .or(get_auction_route);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_mongo_client(
    client: Arc<Client>,
) -> impl Filter<Extract = (Arc<Client>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_config(
    config: Arc<Constants>,
) -> impl Filter<Extract = (Arc<Constants>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_ethers_client(
    client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> impl Filter<
    Extract = (Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || client.clone())
}

#[derive(Debug)]
pub struct ServerError {
    _reason: String,
}

impl From<anyhow::Error> for ServerError {
    fn from(err: anyhow::Error) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl<M> From<ethers::contract::ContractError<M>> for ServerError
where
    M: Middleware,
{
    fn from(err: ethers::contract::ContractError<M>) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<ProviderError> for ServerError {
    fn from(err: ProviderError) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<ethers::contract::AbiError> for ServerError {
    fn from(err: ethers::contract::AbiError) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl warp::reject::Reject for ServerError {}

#[derive(Serialize)]
struct TransactionResponse {
    transaction_hash: H256,
}

#[derive(Serialize)]
struct ContractCallResponse {
    transaction_hash: String,
}

#[derive(Deserialize, Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct EchoResponse {
    message: String,
}

#[derive(Serialize)]
struct POSTResponse {
    success: bool,
    message: String,
}
