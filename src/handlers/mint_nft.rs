use crate::chain::chain::{SendTransactionResult, TransactionReceiptSchema, TxHashSchema};
use crate::chain::mint::mint_nft;
use crate::constants::Constants;
use crate::db::mongo::{add_nft, AddNFTInput, Metadata};
use crate::error::ServerError;
use ethers::types::H256;
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use warp::http::StatusCode;

#[derive(Deserialize, Clone, ToSchema)]
pub struct MintUniqueTokenRequest {
    pub owner_address: String,
    pub token_id: u64,
    pub metadata: Metadata, // Accepting metadata as a structured object
    pub wait_confirmation: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/api/mint",
    request_body = MintUniqueTokenRequest,
    responses(
        (status = 200, description = "Mint NFT successfully", body = MintNFTSuccessResponse), // Define MintNftResponse struct with ToSchema
        (status = 400, description = "Bad Request"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn mint_nft_handler(
    req: MintUniqueTokenRequest,
    mongo_client: Arc<Client>,
    config: Arc<Constants>,
    auth_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    if auth_id == "test" {
        let mock_response = mint_mock_response(req.clone());
        return Ok(warp::reply::with_status(
            warp::reply::json(&mock_response),
            StatusCode::CREATED,
        ));
    }
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

#[derive(Serialize, ToSchema)]
pub struct MintNFTSuccessResponse {
    nft_details: AddNFTInput, // Replace `YourNftDetailsType` with the actual type
    tx_result: SendTransactionResult,
}

fn mint_mock_response(req: MintUniqueTokenRequest) -> MintNFTSuccessResponse {
    let tx_result: SendTransactionResult;
    let wait_confirm_for_mock = req.wait_confirmation.clone().unwrap_or(false);
    if wait_confirm_for_mock {
        let mock_receipt = TransactionReceiptSchema {
            transaction_hash: "0xTRXHASH".to_string(),
            transaction_index: 1,
            block_hash: Some("0xBLOCKHASH".to_string()),
            block_number: Some(1),
            from: "0xTRXSENDER".to_string(),
            to: Some("0xTRXRECEIVER".to_string()),
            cumulative_gas_used: "CUMULATIVEGASUSED".to_string(),
            gas_used: Some("GASUSED".to_string()),
            contract_address: Some("0xCONTRACTADDRESS".to_string()),
            status: Some(0),
            effective_gas_price: Some("GASPRICEinGWEI".to_string()),
        };

        tx_result = SendTransactionResult::Receipt(mock_receipt);
    } else {
        tx_result =
            SendTransactionResult::Hash(TxHashSchema::from(H256::from_low_u64_be(123456789)));
    }

    let token_nft = AddNFTInput {
        token_id: req.token_id,
        metadata: req.metadata,
    };

    let mock_response = MintNFTSuccessResponse {
        nft_details: token_nft.clone(),
        tx_result,
    };
    mock_response
}
