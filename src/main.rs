use chain::chain::send_transaction_with_retry;
use db::mongo::{add_nft, find_nfts, find_one_nft, AddNFTInput, Metadata};
use ethers::abi::Abi;
use ethers::{prelude::*, utils::hex};
use graph::graph::{graphql_owner_tokens_query, graphql_token_owner_query, reqwest_graphql_query};
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::Filter;

mod db {
    pub mod mongo;
}
mod graph {
    pub mod graph;
}
mod chain {
    pub mod chain;
}

// const CONTRACT_ADDRESS: &str = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
const CONTRACT_ADDRESS: &str = "0x8707deaa13aD0883045EC2905BBC22e6d041dC40";
const ABI_PATH: &[u8; 8420] = include_bytes!("abi/SnapitNFT.json");
// const PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const PRIVATE_KEY: &str = "3765ae92eec18f50c24b230a3f0c7c75868f9c898985d1dec97845df1a54fd16";
pub const CHAIN_URL: &str = "https://data-seed-prebsc-1-s3.binance.org:8545"; //http://localhost:8545
const CHAIN_ID: u64 = 97; // 31337; // Explicitly type the chain ID as `u64`
const GRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/basarrcan/snapit-test";
// const GRAPH_URL: &str = "http://localhost:8000/subgraphs/name/basarrcan/firstsubgraph";

#[derive(Deserialize)]
struct QueryParams {
    with_owner: Option<String>,
}

#[tokio::main]
async fn main() {
    let mongo_client = db::mongo::init_db().await.expect("Failed to initialize DB");

    let provider = Provider::<Http>::try_from(CHAIN_URL).unwrap();

    let wallet: LocalWallet = PRIVATE_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(CHAIN_ID); // Set this to the chain ID of your network

    let ethers_client = Arc::new(SignerMiddleware::new(provider, wallet));

    // GET endpoint at /
    let get_route = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::json(&"Welcome to the Rust API!"));

    // POST endpoint at /echo
    let post_route = warp::post()
        .and(warp::path("echo"))
        .and(warp::body::json())
        .map(|body: EchoRequest| {
            warp::reply::json(&EchoResponse {
                message: body.message,
            })
        });

    let mint_nft_client = ethers_client.clone();
    let add_nft_client = mongo_client.clone();
    let mint_nft_route = warp::post()
        .and(warp::path("api"))
        .and(warp::path("mint"))
        .and(warp::body::json())
        .and(warp::any().map(move || mint_nft_client.clone()))
        .and(warp::any().map(move || add_nft_client.clone()))
        .and_then(mint_nft);

    let get_nft_client = mongo_client.clone();

    let get_nft_route = warp::get()
        .and(warp::any().map(move || get_nft_client.clone()))
        .and(warp::path("api"))
        .and(warp::path("token"))
        .and(warp::path::param::<String>()) // Capture {id}.json as a String
        .and(warp::query::<QueryParams>()) // Use query to capture with_owner
        .and_then(get_nft);

    let get_owner_tokens_client = mongo_client.clone();

    let get_owner_tokens_route = warp::get()
        .and(warp::any().map(move || get_owner_tokens_client.clone()))
        .and(warp::path("api"))
        .and(warp::path("get-owner-tokens"))
        .and(warp::query::<GetOwnerTokensQueryParams>())
        .and_then(get_owner_tokens_handler);

    // Combine the routes
    let routes = get_route
        .or(post_route)
        .or(mint_nft_route)
        .or(get_owner_tokens_route)
        .or(get_nft_route);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn mint_nft(
    req: MintUniqueTokenRequest,
    ethers_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    mongo_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let abi: Abi = serde_json::from_slice(ABI_PATH).unwrap();

    let contract_address = Address::from_str(CONTRACT_ADDRESS).unwrap();

    let contract = Contract::new(contract_address, abi, ethers_client);

    let metadata_json = serde_json::to_string(&req.metadata).unwrap();
    let metadata_hex = hex::encode(metadata_json);

    let data_bytes = Bytes::from(hex::decode(metadata_hex).expect("Invalid hex"));

    let owner_address = Address::from_str(&req.owner_address).unwrap();

    let token_id = U256::from(req.token_id);

    let tx_hash = match send_transaction_with_retry(
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
        Err(e) => {
            println!("Error minting NFT on chain: {:?}", e);
            // Properly return a Rejection in case of error

            Err(warp::reject::custom(ServerError))
        }
    };

    println!("{:?}", tx_hash);

    let token_nft = AddNFTInput {
        token_id: req.token_id,
        metadata: req.metadata,
    };

    match add_nft(mongo_client, token_nft).await {
        Ok(()) => Ok(warp::reply::with_status(
            warp::reply::json(&"NFT added successfully"),
            StatusCode::CREATED,
        )),
        Err(e) => {
            println!("Error adding NFT: {:?}", e);
            // Properly return a Rejection in case of error

            Err(warp::reject::custom(ServerError))
        }
    }
}

async fn get_nft(
    client: Client,
    id_json: String, // Ensure this matches the type expected by your MongoDB function
    params: QueryParams,
) -> Result<impl warp::Reply, warp::Rejection> {
    let with_owner = params.with_owner.map(|v| v == "true").unwrap_or(false);

    let id_str = id_json.trim_end_matches(".json");
    let query = graphql_token_owner_query(id_str);

    // Attempt to parse the numeric part as u64
    match id_str.parse::<u64>() {
        Ok(token_id) => {
            // If parsing succeeds, proceed with your logic using `token_id`
            match find_one_nft(client, token_id as u64).await {
                // Cast to u64 if needed
                Ok(Some(mut metadata)) => {
                    if with_owner {
                        let res = reqwest_graphql_query(query, GRAPH_URL).await?;

                        let token_balances = res["data"]["tokenBalances"]
                            .as_array()
                            .ok_or("Invalid response format")
                            .map_err(|_| warp::reject::custom(ServerError))?;

                        let owner_address: &str = match token_balances.get(0) {
                            Some(tb) => tb["owner"].as_str().unwrap_or("default"),
                            None => "default",
                        };

                        if let Some(metadata_map) = metadata.as_object_mut() {
                            metadata_map.insert(
                                "owner".to_string(),
                                serde_json::Value::String(owner_address.to_string()),
                            );
                        }
                    }
                    Ok(warp::reply::with_status(
                        warp::reply::json(&metadata),
                        StatusCode::OK,
                    ))
                }
                Ok(None) => Ok(warp::reply::with_status(
                    warp::reply::json(&"NFT not found"),
                    StatusCode::NOT_FOUND,
                )),
                Err(e) => {
                    println!("Error fetching NFT metadata: {:?}", e);
                    Err(warp::reject::custom(ServerError))
                }
            }
        }
        Err(_) => {
            // If parsing fails, reject the request
            Err(warp::reject::not_found())
        }
    }
}

async fn get_owner_tokens_handler(
    client: Client,
    params: GetOwnerTokensQueryParams,
) -> Result<impl warp::Reply, warp::Rejection> {
    let owner_address = params.owner_address;
    let query = graphql_owner_tokens_query(&owner_address);

    let res = reqwest_graphql_query(query, GRAPH_URL).await?;

    // reqwest_client
    //     .post(GRAPH_URL)
    //     .json(&serde_json::json!({"query": query}))
    //     .send()
    //     .await
    //     .map_err(|_| warp::reject::custom(ServerError))?
    //     .json::<Value>()
    //     .await
    //     .map_err(|_| warp::reject::custom(ServerError))?; // Handle HTTP request error

    let token_balances = res["data"]["tokenBalances"]
        .as_array()
        .ok_or("Invalid response format")
        .map_err(|_| warp::reject::custom(ServerError))?;

    // Extract token IDs from token_balances
    let token_ids: Vec<u64> = token_balances
        .iter()
        .filter_map(|tb| tb["token"]["id"].as_str())
        .filter_map(|id| id.parse::<u64>().ok())
        .collect();

    // Call find_nfts with the extracted token IDs
    let nfts = find_nfts(client, token_ids)
        .await
        .map_err(|_| warp::reject::custom(ServerError))?;

    let transformed: Vec<Value> = nfts
        .iter()
        // .filter_map(|tb| tb["token"].as_object())
        .map(|nft| {
            serde_json::json!({
                "token_id": nft["token_id"],
                "metadata": nft["metadata"]
            })
        })
        .collect();

    // let response_body = transformed
    //     .text()
    //     .await
    //     .map_err(|_| warp::reject::custom(ServerError))?; // Handle response error

    let json_reply = warp::reply::json(&transformed);

    Ok(warp::reply::with_status(json_reply, StatusCode::OK))
}

#[derive(Debug)]
struct ServerError;
impl warp::reject::Reject for ServerError {}

#[derive(Deserialize)]
struct MintUniqueTokenRequest {
    owner_address: String,
    token_id: u64,
    metadata: Metadata, // Accepting metadata as a structured object
}

#[derive(Deserialize)]
struct GetOwnerTokensQueryParams {
    owner_address: String,
}

#[derive(Serialize)]
struct TransactionResponse {
    transaction_hash: H256,
}

#[derive(Serialize)]
struct ContractCallResponse {
    transaction_hash: String,
}

#[derive(Deserialize, Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct EchoResponse {
    message: String,
}

#[derive(Serialize)]
struct POSTResponse {
    success: bool,
    message: String,
}
