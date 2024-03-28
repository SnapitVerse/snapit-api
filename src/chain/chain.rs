use std::sync::Arc;

use anyhow::Result; // Simplified error handling with anyhow

use ethers::contract::FunctionCall;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::gas_escalator::{Frequency, GeometricGasPrice};
// use ethers::middleware::gas_oracle::{
//     EthGasStation, Etherchain, Etherscan, GasCategory, GasNow, GasOracleMiddleware,
// };
use ethers::middleware::{GasEscalatorMiddleware, MiddlewareBuilder, NonceManagerMiddleware};

use ethers::signers::{LocalWallet, Signer};

use ethers::types::H256;
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use warp::Filter;

use crate::constants::Constants;
use crate::error::ServerError;

// type EthersClient = NonceManagerMiddleware<
//     SignerMiddleware<
//         GasOracleMiddleware<GasEscalatorMiddleware<Provider<Http>>, GasNow>,
//         Wallet<SigningKey>,
//     >,
// >;

// type EthersContract = NonceManagerMiddleware<
//     SignerMiddleware<
//         GasOracleMiddleware<GasEscalatorMiddleware<Provider<Http>>, GasNow>,
//         Wallet<SigningKey>,
//     >,
// >;

type EthersClient = NonceManagerMiddleware<
    SignerMiddleware<
        //GasOracleMiddleware<
        GasEscalatorMiddleware<Provider<Http>>,
        //GasNow>,
        Wallet<SigningKey>,
    >,
>;

type EthersContract = NonceManagerMiddleware<
    SignerMiddleware<
        // GasOracleMiddleware <
        GasEscalatorMiddleware<Provider<Http>>,
        //>, GasNow>,
        Wallet<SigningKey>,
    >,
>;

pub type EthersProvider = Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

// type EthersContractInstance = ContractInstance<Arc<EthersContract>, EthersContract>;

type EthersContractCall = FunctionCall<Arc<EthersContract>, EthersContract, H256>;

pub async fn get_ethers_client() -> Result<Arc<EthersClient>> {
    let config = Constants::new();

    let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
    let signer = config.private_key.parse::<LocalWallet>().unwrap();
    let provider = Provider::<Http>::try_from(config.chain_url.as_str()).unwrap();

    let signer_address = signer.address();

    // let category = GasCategory::Standard;
    // let oracle = GasNow::new().category(category);

    let provider =
        provider.wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock));
    // .gas_oracle(oracle);
    let provider = SignerMiddleware::new_with_provider_chain(provider, signer)
        .await
        .unwrap();
    let provider = provider.nonce_manager(signer_address);
    Ok(Arc::new(provider))
}

pub fn with_ethers_client(
    client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> impl Filter<
    Extract = (Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || client.clone())
}

pub async fn send_transaction(
    contract_call: EthersContractCall,
    wait_confirmation: bool,
) -> Result<SendTransactionResult, ServerError> {
    // Try directly calling the method and setting the nonce without intermediate binding
    match contract_call.send().await {
        Ok(pending_tx) => {
            let tx_result = pending_tx;
            if wait_confirmation {
                let tx_result = tx_result.await?.unwrap();
                return Ok(SendTransactionResult::Receipt(tx_result.into()));
            }

            // pending_tx.
            Ok(SendTransactionResult::Hash(tx_result.tx_hash().into()))
        }
        Err(e) => Err(ServerError::from(e)),
    }
}

#[derive(Serialize, ToSchema)]
pub enum SendTransactionResult {
    Receipt(TransactionReceiptSchema),
    Hash(TxHashSchema),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TransactionReceiptSchema {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: u64,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<String>,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<u64>,
    /// address of the sender.
    pub from: String,
    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<String>,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: String,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<u64>,
    #[serde(
        rename = "effectiveGasPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub effective_gas_price: Option<String>,
}

impl From<ethers::types::TransactionReceipt> for TransactionReceiptSchema {
    fn from(receipt: ethers::types::TransactionReceipt) -> Self {
        TransactionReceiptSchema {
            transaction_hash: receipt.transaction_hash.to_string(),
            transaction_index: receipt.transaction_index.as_u64(),
            block_hash: receipt.block_hash.map(|hash| hash.to_string()),
            block_number: receipt.block_number.map(|num| num.as_u64()),
            from: receipt.from.to_string(),
            to: receipt.to.map(|to| to.to_string()),
            cumulative_gas_used: receipt.cumulative_gas_used.to_string(),
            gas_used: receipt.gas_used.map(|gas| gas.to_string()),
            contract_address: receipt.contract_address.map(|address| address.to_string()),
            status: receipt.status.map(|status| status.as_u64()),
            effective_gas_price: receipt.effective_gas_price.map(|price| price.to_string()),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct TxHashSchema(pub [u8; 32]);

impl From<H256> for TxHashSchema {
    fn from(hash: H256) -> Self {
        TxHashSchema(hash.0)
    }
}
