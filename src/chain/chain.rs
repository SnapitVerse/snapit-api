use anyhow::{Error, Result}; // Simplified error handling with anyhow
use async_recursion::async_recursion;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::providers::Middleware;
use ethers::types::Bytes;
use ethers::{
    contract::Contract,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H256, U256},
};

#[async_recursion]
pub async fn send_transaction_with_retry(
    chain_url: &str,
    contract: &Contract<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    owner_address: Address,
    token_id: U256,
    data_bytes: Bytes,
    nonce: Option<U256>,
    gas_price: Option<U256>,
    retries: Option<u32>,
) -> Result<H256> {
    let retry = retries.unwrap_or(0);
    if retry >= 5 {
        return Err(Error::msg("Max retries exceeded").into());
    }

    let provider = Provider::<Http>::try_from(chain_url).unwrap();

    let mut contract_call = contract.method::<_, H256>(
        "mintUniqueToken",
        (owner_address, token_id, data_bytes.clone()),
    )?;

    if let Some(nonce_option) = nonce {
        contract_call = contract_call.nonce(nonce_option)
    }

    let gas_price_from_chain = match gas_price {
        Some(price) => price,
        None => provider.get_gas_price().await?,
    };
    contract_call = contract_call.gas_price(gas_price_from_chain);

    // let tx = match

    // Try directly calling the method and setting the nonce without intermediate binding
    let sent_trx = match contract_call.send().await {
        Ok(pending_tx) => Ok(pending_tx.tx_hash()),
        Err(e) => {
            if let Some(new_nonce) = extract_nonce_from_error(&e.to_string()) {
                // Recursively retry with the updated nonce
                send_transaction_with_retry(
                    chain_url,
                    contract,
                    owner_address,
                    token_id,
                    data_bytes, // Consider cloning if needed
                    Some(new_nonce),
                    Some(gas_price_from_chain),
                    Some(retry + 1),
                )
                .await
            } else if is_underpriced_error(&e.to_string()) {
                send_transaction_with_retry(
                    chain_url,
                    contract,
                    owner_address,
                    token_id,
                    data_bytes, // Consider cloning if needed
                    nonce,
                    Some(increase_gas_price_by_10_percent(gas_price_from_chain)),
                    Some(retry + 1),
                )
                .await
            } else {
                Err(anyhow::Error::from(e).into())
            }
        }
    };
    sent_trx
}

fn extract_nonce_from_error(error_message: &str) -> Option<U256> {
    // Look for the pattern that precedes the nonce value in the error message
    let nonce_pattern = "next nonce ";
    if let Some(start_index) = error_message.find(nonce_pattern) {
        // Calculate the start position of the nonce value
        let nonce_start = start_index + nonce_pattern.len();
        // Extract the substring starting from the nonce value
        let nonce_substring = &error_message[nonce_start..];

        // Use whitespace or comma as potential delimiters for the end of the nonce value
        let nonce_end = nonce_substring
            .find(|c: char| c.is_whitespace() || c == ',')
            .unwrap_or(nonce_substring.len());

        // Extract the nonce value as a string
        let nonce_str = &nonce_substring[..nonce_end];

        // Attempt to parse the nonce value as a U256
        match nonce_str.parse::<u64>() {
            Ok(nonce_value) => Some(U256::from(nonce_value)),
            Err(_) => None, // Return None if parsing fails
        }
    } else {
        // If the pattern is not found, return None
        None
    }
}

fn is_underpriced_error(error_message: &str) -> bool {
    // Look for the pattern that precedes the nonce value in the error message
    let pattern = "replacement transaction underpriced";
    if let Some(_) = error_message.find(pattern) {
        return true;
    }
    false
}

fn increase_gas_price_by_10_percent(gas_price: U256) -> U256 {
    // Calculate 10% of the current gas price
    let ten_percent = gas_price / U256::from(10);

    // Add the 10% increase to the original gas price
    let increased_gas_price = gas_price + ten_percent;

    increased_gas_price
}
