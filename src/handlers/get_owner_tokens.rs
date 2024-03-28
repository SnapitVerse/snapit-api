use crate::constants::Constants;
use crate::db::mongo::find_nfts;
use crate::error::ServerError;
use crate::graph::graph::{graphql_owner_tokens_query, reqwest_graphql_query};
use std::sync::Arc;

use anyhow::anyhow;
use mongodb::Client;
use serde::Deserialize;
use serde_json::{self, Value};
use utoipa::IntoParams;
use warp::http::StatusCode;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetOwnerTokensQueryParams {
    owner_address: String,
}

#[utoipa::path(
    get,
    path = "/api/owner-tokens",
    params(GetOwnerTokensQueryParams),
    responses(
        (status = 200, description = "Returns all NFTs owned by address", body = [Value])
    )
)]
pub async fn get_owner_tokens_handler(
    client: Arc<Client>,
    params: GetOwnerTokensQueryParams,
    config: Arc<Constants>,
    _auth_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let owner_address = params.owner_address;
    let query = graphql_owner_tokens_query(&owner_address);

    let res = reqwest_graphql_query(query, config.graph_url_nft.as_str()).await?;

    let token_balances = res["data"]["tokenOwnerships"]
        .as_array()
        .ok_or("Invalid response format")
        .map_err(|e| warp::reject::custom(ServerError::from(anyhow!(e))))?;

    // Extract token IDs from token_balances
    let token_ids: Vec<u64> = token_balances
        .iter()
        .filter_map(|tb| tb["token"]["id"].as_str())
        .filter_map(|id| id.parse::<u64>().ok())
        .collect();

    // Call find_nfts with the extracted token IDs
    let nfts = find_nfts(client, token_ids)
        .await
        .map_err(|e| warp::reject::custom(ServerError::from(e)))?;

    let transformed: Vec<Value> = nfts
        .iter()
        // .filter_map(|tb| tb["token"].as_object())
        .map(|nft| {
            serde_json::json!({
                "token_id": nft.token_id,
                "metadata": nft.metadata
            })
        })
        .collect();

    let json_reply = warp::reply::json(&transformed);

    Ok(warp::reply::with_status(json_reply, StatusCode::OK))
}
