use crate::chain::chain::SendTransactionResult;
use crate::chain::mint::mint_nft;
use crate::constants::Constants;
use crate::db::mongo::{add_nft, AddNFTInput, Metadata};
use crate::ServerError;
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::http::StatusCode;

#[derive(Deserialize, Clone)]
pub struct MintUniqueTokenRequest {
    pub owner_address: String,
    pub token_id: u64,
    pub metadata: Metadata, // Accepting metadata as a structured object
    pub wait_confirmation: Option<bool>,
}

pub async fn mint_nft_handler(
    req: MintUniqueTokenRequest,
    mongo_client: Arc<Client>,
    config: Arc<Constants>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let tx_result = mint_nft(req.clone(), config)
        .await
        .map_err(|e| warp::reject::custom(ServerError::from(e)))?;

    let token_nft = AddNFTInput {
        token_id: req.token_id,
        metadata: req.metadata,
    };

    match add_nft(mongo_client, token_nft.clone()).await {
        Ok(()) => {
            let success_response = MintNFTSuccessResponse {
                nft_details: token_nft,
                tx_result,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&success_response),
                StatusCode::CREATED,
            ))
        }
        Err(e) => Err(warp::reject::custom(ServerError::from(e))),
    }
}

#[derive(Serialize)]
struct MintNFTSuccessResponse {
    nft_details: AddNFTInput, // Replace `YourNftDetailsType` with the actual type
    tx_result: SendTransactionResult,
}
