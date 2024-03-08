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
use ethers::types::TransactionReceipt;
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::H256,
};
use serde::Serialize;

use crate::constants::Constants;
use crate::ServerError;

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
                return Ok(SendTransactionResult::Receipt(tx_result));
            }

            // pending_tx.
            Ok(SendTransactionResult::Hash(tx_result.tx_hash()))
        }
        Err(e) => Err(ServerError::from(e)),
    }
}

#[derive(Serialize)]
pub enum SendTransactionResult {
    Receipt(TransactionReceipt),
    Hash(ethers::types::TxHash),
}
