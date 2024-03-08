use crate::graph::graph::{graphql_auction_bid_query, reqwest_graphql_query};
use anyhow::anyhow;
use ethers::prelude::*;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;

use crate::constants::Constants;
use crate::ServerError;

#[derive(Deserialize)]
pub struct GetAuctionQueryParams {
    token_id: u64,
}

abigen!(
    AuctionContract,
    "src/abi/Auction.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub async fn get_auction(
    // client: Arc<Client>,
    ethers_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    params: GetAuctionQueryParams,
    config: Arc<Constants>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let contract_address = Address::from_str(config.auction_address.as_str()).unwrap();

    let contract = AuctionContract::new(contract_address, ethers_client);

    match contract.auctions(U256::from(params.token_id)).await {
        Ok(auction_data_tuple) => {
            let auction_data = AuctionData {
                auction_owner: auction_data_tuple.0,
                min_price_difference: auction_data_tuple.1,
                start_time: auction_data_tuple.2,
                end_time: auction_data_tuple.3,
                buyout_price: auction_data_tuple.4,
                bid_owner: auction_data_tuple.5,
                bid_price: auction_data_tuple.6,
                claimed: auction_data_tuple.7,
            };

            if auction_data.auction_owner
                == H160::from_str("0x0000000000000000000000000000000000000000").unwrap()
            {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&"Auction not found!"),
                    StatusCode::OK,
                ));
            }

            let token_id_str = params.token_id.to_string();
            let start_time_str = auction_data.start_time.to_string();
            let end_time_str = auction_data.end_time.to_string();

            let query = graphql_auction_bid_query(
                token_id_str.as_str(),
                start_time_str.as_str(),
                end_time_str.as_str(),
            );

            let res = reqwest_graphql_query(query, config.graph_url_auction.as_str()).await?;

            let bids = res["data"]["bids"]
                .as_array()
                .ok_or("Invalid response format")
                .map_err(|e| warp::reject::custom(ServerError::from(anyhow!(e))))?;

            let bid_history: Vec<Bid> = bids
                .iter()
                .map(|bid_value| {
                    // Attempt to deserialize each serde_json::Value into a Bid
                    serde_json::from_value(bid_value.clone()).expect("Failed to deserialize Bid")
                })
                .map(|camel_value| convert_camel_to_snake_bid(camel_value))
                .collect();

            let auction_result = GetAuctionResult {
                auction_data,
                bid_history,
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&auction_result),
                StatusCode::OK,
            ))
        }
        Err(e) => Err(warp::reject::custom(ServerError::from(e))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct AuctionData {
    auction_owner: Address,
    min_price_difference: U256,
    start_time: U256,
    end_time: U256,
    buyout_price: U256,
    bid_owner: Address,
    bid_price: U256,
    claimed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphResultBid {
    #[serde(rename = "tokenId", deserialize_with = "deserialize_string_to_u64")]
    token_id: u64,

    #[serde(rename = "price")]
    price: String,

    #[serde(rename = "bidder")]
    bidder: String,

    #[serde(
        rename = "blockTimestamp",
        deserialize_with = "deserialize_string_to_u64"
    )]
    block_timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Bid {
    token_id: u64,
    price: String,
    bidder: String,
    block_timestamp: u64,
}

fn convert_camel_to_snake_bid(camel_case_data: GraphResultBid) -> Bid {
    Bid {
        token_id: camel_case_data.token_id,
        price: camel_case_data.price,
        bidder: camel_case_data.bidder,
        block_timestamp: camel_case_data.block_timestamp,
    }
}

// Helper function to deserialize a stringified number into a u64
fn deserialize_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(DeError::custom)
}

#[derive(Serialize, Deserialize)]
struct GetAuctionResult {
    auction_data: AuctionData,
    bid_history: Vec<Bid>,
}
