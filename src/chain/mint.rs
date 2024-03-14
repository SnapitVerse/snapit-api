use crate::constants::Constants;
use crate::handlers::mint_nft::MintUniqueTokenRequest;
use crate::ServerError;
use ethers::abi::{Abi, Address};
use ethers::prelude::*;
use serde_json::{self};
use std::str::FromStr;
use std::sync::Arc;

use super::chain::{get_ethers_client, send_transaction, SendTransactionResult};
use super::helpers::object_to_data_bytes; // Import the `mongo` module

const ABI_PATH: &[u8; 8420] = include_bytes!("../abi/SnapitNFT.json");

pub async fn mint_nft(
    req: MintUniqueTokenRequest,
    config: Arc<Constants>,
) -> Result<SendTransactionResult, ServerError> {
    let abi: Abi = serde_json::from_slice(ABI_PATH).unwrap();
    let ethers_client = get_ethers_client().await?;

    let contract_address = Address::from_str(config.nft_address.as_str()).unwrap();
    let contract = Contract::new(contract_address, abi, ethers_client);

    let data_bytes = object_to_data_bytes(req.metadata);
    let owner_address = Address::from_str(&req.owner_address).unwrap();
    let token_id = U256::from(req.token_id);

    let contract_call = contract.method::<_, H256>(
        "mint",
        (owner_address, token_id, data_bytes.clone()),
    )?;

    let wait_confirmation = req.wait_confirmation.unwrap_or(true);

    match send_transaction(contract_call, wait_confirmation).await {
        Ok(tx_receipt) => Ok(tx_receipt),
        Err(e) => Err(ServerError::from(e)),
    }
}
