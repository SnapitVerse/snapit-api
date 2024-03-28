use serde::Deserialize;
use utoipa::IntoParams;

use std::sync::Arc;
use warp::http::StatusCode;

use crate::alchemy::alchemy::{alchemy_nft_sales_request, AlchemyNftSalesEndpointQueryParams};
use crate::constants::Constants;
use crate::error::ServerError;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetNFTMarketSalesQueryParams {
    token_id: Option<u64>,
    page_key: Option<String>,
    limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/nft-sales",
    params(GetNFTMarketSalesQueryParams),
    responses(
        (status = 200, description = "Returns NFT Market sale info", body = [Value])
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_nft_sales_handler(
    // client: Arc<Client>,
    params: GetNFTMarketSalesQueryParams,
    config: Arc<Constants>,
    _auth_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let contract_deploy_block: String = "5484602".to_string();

    let result_limit = params.limit.unwrap_or(10);

    let token_id: Option<String> = match params.token_id {
        Some(id) => Some(id.to_string()),
        None => None,
    };

    let query_params = AlchemyNftSalesEndpointQueryParams {
        from_block: Some(contract_deploy_block),
        order: Some("desc".to_string()),
        marketplace: "seaport".to_string(),
        // contract_address: config.auction_address.clone(),
        contract_address: None,
        token_id,
        buyer_address: None,
        seller_address: None,
        limit: Some(result_limit),
        page_key: params.page_key,
    };

    match alchemy_nft_sales_request(query_params, config).await {
        Ok(response) => Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::OK,
        )),
        Err(e) => Err(warp::reject::custom(ServerError::from(e))),
    }
}
