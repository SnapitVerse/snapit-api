use crate::db::mongo::{contract_metadata, find_one_nft};
use crate::graph::graph::{graphql_token_owner_query, reqwest_graphql_query};
use anyhow::anyhow;
use mongodb::Client;
use serde::Deserialize;
use serde_json::{self};
use std::sync::Arc;
use warp::http::StatusCode;

use crate::constants::Constants;
use crate::ServerError;

#[derive(Deserialize)]
pub struct GetNftQueryParams {
    with_owner: Option<String>,
}

pub async fn get_nft(
    client: Arc<Client>,
    id_json: String, // Ensure this matches the type expected by your MongoDB function
    params: GetNftQueryParams,
    config: Arc<Constants>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let with_owner = params.with_owner.map(|v| v == "true").unwrap_or(false);

    let id_str = id_json.trim_end_matches(".json");

    if id_str == "contract-metadata" {
        return match contract_metadata(client.clone()).await {
            Ok(Some(metadata)) => Ok(warp::reply::with_status(
                warp::reply::json(&metadata),
                StatusCode::OK,
            )),
            Ok(None) => Err(warp::reject::custom(ServerError::from(anyhow!(
                "Contract metadata not found"
            )))),
            Err(e) => Err(warp::reject::custom(ServerError::from(e))),
        };
    }

    // Attempt to parse the numeric part as u64
    match id_str.parse::<u64>() {
        Ok(token_id) => {
            // If parsing succeeds, proceed with your logic using `token_id`
            match find_one_nft(client, token_id as u64).await {
                // Cast to u64 if needed
                Ok(Some(mut metadata)) => {
                    if with_owner {
                        let query = graphql_token_owner_query(id_str);

                        let res =
                            reqwest_graphql_query(query, config.graph_url_token.as_str()).await?;

                        let token_balances = res["data"]["tokenBalances"]
                            .as_array()
                            .ok_or("Invalid response format")
                            .map_err(|e| warp::reject::custom(ServerError::from(anyhow!(e))))?;

                        let owner_address: &str = match token_balances.get(0) {
                            Some(tb) => tb["owner"].as_str().unwrap_or("default"),
                            None => "default",
                        };

                        if let Some(metadata_map) = metadata.as_object_mut() {
                            metadata_map.insert(
                                "owner".to_string(),
                                serde_json::Value::String(owner_address.to_string()),
                            );
                        }
                    }
                    Ok(warp::reply::with_status(
                        warp::reply::json(&metadata),
                        StatusCode::OK,
                    ))
                }
                Ok(None) => Ok(warp::reply::with_status(
                    warp::reply::json(&"NFT not found"),
                    StatusCode::NOT_FOUND,
                )),
                Err(e) => Err(warp::reject::custom(ServerError::from(e))),
            }
        }
        Err(_) => {
            // If parsing fails, reject the request
            Err(warp::reject::not_found())
        }
    }
}
