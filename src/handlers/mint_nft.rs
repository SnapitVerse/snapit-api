use crate::chain::chain::send_transaction_with_retry;
use crate::constants::Constants;
use crate::db::mongo::{add_nft, AddNFTInput, Metadata};
use crate::ServerError;
use ethers::abi::Abi;
use ethers::{prelude::*, utils::hex};
use mongodb::bson::doc;
use mongodb::Client;
use serde::Deserialize;
use serde_json::{self};
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode; // Import the `mongo` module

#[derive(Deserialize)]
pub struct MintUniqueTokenRequest {
    owner_address: String,
    token_id: u64,
    metadata: Metadata, // Accepting metadata as a structured object
}

const ABI_PATH: &[u8; 8420] = include_bytes!("../abi/SnapitNFT.json");

pub async fn mint_nft(
    req: MintUniqueTokenRequest,
    ethers_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    mongo_client: Arc<Client>,
    config: Arc<Constants>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let abi: Abi = serde_json::from_slice(ABI_PATH).unwrap();

    let contract_address = Address::from_str(config.nft_address.as_str()).unwrap();

    let contract = Contract::new(contract_address, abi, ethers_client);

    let metadata_json = serde_json::to_string(&req.metadata).unwrap();
    let metadata_hex = hex::encode(metadata_json);

    let data_bytes = Bytes::from(hex::decode(metadata_hex).expect("Invalid hex"));

    let owner_address = Address::from_str(&req.owner_address).unwrap();

    let token_id = U256::from(req.token_id);

    let _tx_hash = match send_transaction_with_retry(
        config.chain_url.as_str(),
        &contract,
        owner_address,
        token_id,
        data_bytes, // Convert Bytes to Vec<u8>
        None,
        None,
        None,
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(_e) => Err(warp::reject::custom(ServerError)),
    };

    let token_nft = AddNFTInput {
        token_id: req.token_id,
        metadata: req.metadata,
    };

    match add_nft(mongo_client, token_nft).await {
        Ok(()) => Ok(warp::reply::with_status(
            warp::reply::json(&"NFT added successfully"),
            StatusCode::CREATED,
        )),
        Err(_e) => Err(warp::reject::custom(ServerError)),
    }
}
