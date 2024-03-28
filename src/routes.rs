use mongodb::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use warp::{self, Filter};

use crate::auth::with_auth;
use crate::chain::chain::{with_ethers_client, EthersProvider};
use crate::constants::{with_config, Constants};

use crate::db::mongo::with_mongo_client;
use crate::error::handle_rejection;
use crate::openapi::OpenAPIRoutes;
use std::convert::Infallible;
use std::sync::Arc;

use crate::handlers::get_auction::{get_auction, GetAuctionQueryParams};
use crate::handlers::get_nft::{get_nft_handler, GetNftQueryParams};
use crate::handlers::get_nft_sales::{get_nft_sales_handler, GetNFTMarketSalesQueryParams};
use crate::handlers::get_owner_tokens::{get_owner_tokens_handler, GetOwnerTokensQueryParams};
use crate::handlers::mint_nft::mint_nft_handler;

// Define a function that constructs and returns all routes
pub fn routes(
    config: Arc<Constants>,
    mongo_client: Arc<Client>,
    ethers_client: EthersProvider,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    let config_filter = with_config(config);
    let mongo_client_filter = with_mongo_client(mongo_client);
    let ethers_client_filter = with_ethers_client(ethers_client);
    // GET endpoint at /
    let get_route = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::json(&"Welcome to the SnapitWorld API!"));

    // POST endpoint at /echo
    let post_route = warp::post()
        .and(warp::path("api"))
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
        .and(with_auth())
        .and_then(mint_nft_handler);

    let get_nft_route = warp::get()
        .and(mongo_client_filter.clone())
        .and(warp::path("api"))
        .and(warp::path("token"))
        .and(warp::path::param::<String>()) // Capture {id}.json as a String
        .and(warp::query::<GetNftQueryParams>()) // Use query to capture with_owner
        .and(config_filter.clone())
        .and(with_auth())
        .and_then(get_nft_handler);

    let get_owner_tokens_route = warp::get()
        .and(mongo_client_filter.clone())
        .and(warp::path("api"))
        .and(warp::path("owner-tokens"))
        .and(warp::query::<GetOwnerTokensQueryParams>())
        .and(config_filter.clone())
        .and(with_auth())
        .and_then(get_owner_tokens_handler);

    let get_auction_route = warp::get()
        .and(warp::path("api"))
        .and(warp::path("auction"))
        .and(ethers_client_filter.clone())
        .and(warp::query::<GetAuctionQueryParams>()) // Use query to capture with_owner
        .and(config_filter.clone())
        .and(with_auth())
        .and_then(get_auction);

    let get_nft_sales_route = warp::get()
        .and(warp::path("api"))
        .and(warp::path("nft-sales"))
        .and(warp::query::<GetNFTMarketSalesQueryParams>()) // Use query to capture with_owner
        .and(config_filter.clone())
        .and(with_auth())
        .and_then(get_nft_sales_handler);

    let openapi_json_route = OpenAPIRoutes::openapi_json();
    let swagger_ui_route = OpenAPIRoutes::swagger_ui();

    // Combine the routes
    let routes = get_route
        .or(post_route)
        .or(mint_nft_route)
        .or(get_owner_tokens_route)
        .or(get_nft_route)
        .or(get_nft_sales_route)
        .or(get_auction_route)
        .or(openapi_json_route)
        .or(swagger_ui_route)
        .recover(handle_rejection);

    routes
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct EchoRequest {
    message: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct EchoResponse {
    message: String,
}
