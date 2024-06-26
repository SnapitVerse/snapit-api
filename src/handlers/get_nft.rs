use crate::db::mongo::{contract_metadata, find_one_nft, MetadataAttribute};
use crate::graph::graph::{graphql_token_owner_query, reqwest_graphql_query};
use anyhow::anyhow;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToResponse, ToSchema};
use warp::http::StatusCode;

use crate::constants::Constants;
use crate::error::ServerError;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetNftQueryParams {
    with_id: Option<String>,
    with_owner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToResponse, ToSchema)]
pub struct GetNFTResult {
    token_id: Option<String>,
    owner: Option<String>,
    name: String,
    description: String,
    image: String,
    external_url: String,
    attributes: Vec<MetadataAttribute>, // Add other fields as necessary
}

#[utoipa::path(
    get,
    path = "/api/token/{id}.json",
    params(
        ("id" = i32, Path, description = "NFT ID"),
        GetNftQueryParams
    ),
    responses(
        (status = 200, description = "Returns NFT Detail", body = [GetNFTResult])
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_nft_handler(
    client: Arc<Client>,
    id_json: String, // Ensure this matches the type expected by your MongoDB function
    params: GetNftQueryParams,
    config: Arc<Constants>,
    _auth_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let with_id = params.with_id.map(|v| v == "true").unwrap_or(false);
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
                Ok(Some(token)) => {
                    let mut get_nft_result = GetNFTResult {
                        token_id: None,
                        owner: None,
                        name: token.metadata.name,
                        description: token.metadata.description,
                        image: token.metadata.image,
                        external_url: token.metadata.external_url,
                        attributes: token.metadata.attributes,
                    };
                    if with_id {
                        get_nft_result.token_id = Some(token_id.to_string());
                    }

                    if with_owner {
                        let query = graphql_token_owner_query(id_str);

                        let res =
                            reqwest_graphql_query(query, config.graph_url_nft.as_str()).await?;

                        let token_balances = res["data"]["tokenOwnerships"]
                            .as_array()
                            .ok_or("Invalid response format")
                            .map_err(|e| warp::reject::custom(ServerError::from(anyhow!(e))))?;

                        let owner_address: &str = match token_balances.get(0) {
                            Some(tb) => tb["owner"].as_str().unwrap_or("default"),
                            None => "default",
                        };

                        get_nft_result.owner = Some(owner_address.to_string());
                    }
                    Ok(warp::reply::with_status(
                        warp::reply::json(&get_nft_result),
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
