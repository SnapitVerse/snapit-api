use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::{constants::Constants, error::ServerError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlchemyNftSalesEndpointQueryParams {
    pub from_block: Option<String>,
    pub order: Option<String>,
    pub marketplace: String,
    pub contract_address: Option<String>,
    pub token_id: Option<String>,
    pub buyer_address: Option<String>,
    pub seller_address: Option<String>,
    pub limit: Option<u32>,
    pub page_key: Option<String>,
}

pub async fn alchemy_nft_sales_request(
    query: AlchemyNftSalesEndpointQueryParams,
    config: Arc<Constants>,
) -> Result<Value, ServerError> {
    let alchemy_chain = "eth-mainnet";

    let endpoint_url = format!(
        "https://{}.g.alchemy.com/nft/v3/{}/getNFTSales",
        alchemy_chain, config.alchemy_api_key
    );

    let client = reqwest::Client::new();
    let res = client
        .get(endpoint_url)
        .query(&query)
        .send()
        .await
        .map_err(|e| ServerError::from(e))?
        .json::<Value>()
        .await
        .map_err(|e| ServerError::from(e))?;
    Ok(res)
}
