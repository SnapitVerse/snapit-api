use db::mongo::{add_nft, get_nft, Metadata, NFT};
use ethers::abi::Abi;
use ethers::{prelude::*, utils::hex};
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::Filter;

mod db {
    pub mod mongo;
}

const CONTRACT_ADDRESS: &str = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
const ABI_PATH: &[u8; 8420] = include_bytes!("abi/SnapitNFT.json");
const PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

#[tokio::main]
async fn main() {
    let mongo_client = db::mongo::init_db().await.expect("Failed to initialize DB");

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let chain_id: u64 = 31337; // Explicitly type the chain ID as `u64`
    let wallet: LocalWallet = PRIVATE_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id); // Set this to the chain ID of your network

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
        .and(warp::path("mint_nft"))
        .and(warp::body::json())
        .and(warp::any().map(move || mint_nft_client.clone()))
        .and(warp::any().map(move || add_nft_client.clone()))
        .and_then(mint_nft);

    let get_nft_client = mongo_client.clone();
    let get_nft_metadata_route = warp::get()
        .and(warp::path("get_nft_metadata"))
        .and(warp::any().map(move || get_nft_client.clone()))
        .and(warp::path::param())
        .and_then(get_nft_metadata);

    // Combine the routes
    let routes = get_route
        .or(post_route)
        .or(mint_nft_route)
        .or(get_nft_metadata_route);

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

    let contract_call = contract
        .method::<_, H256>("mintUniqueToken", (owner_address, token_id, data_bytes))
        .unwrap();

    let pending_tx = contract_call.send().await.unwrap();

    let tx_hash = pending_tx.tx_hash();

    let token_nft = NFT {
        token_id: req.token_id,
        metadata: req.metadata,
    };

    // Create a response object
    let _response = TransactionResponse {
        transaction_hash: tx_hash,
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

async fn get_nft_metadata(
    client: Client,
    token_id: u64, // Ensure this matches the type expected by your MongoDB function
) -> Result<impl warp::Reply, warp::Rejection> {
    match get_nft(client, token_id as u64).await {
        // Cast to u64 if needed
        Ok(Some(metadata)) => Ok(warp::reply::with_status(
            warp::reply::json(&metadata),
            StatusCode::OK,
        )),
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

#[derive(Debug)]
struct ServerError;
impl warp::reject::Reject for ServerError {}

#[derive(Deserialize)]
struct MintUniqueTokenRequest {
    owner_address: String,
    token_id: u64,
    metadata: Metadata, // Accepting metadata as a structured object
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

// struct TokenInfoRequest {
//     token_id: U256,
// }

#[derive(Serialize)]
struct POSTResponse {
    success: bool,
    message: String,
}
