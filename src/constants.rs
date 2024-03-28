use std::{env, sync::Arc};

use warp::Filter;

pub struct Constants {
    pub nft_address: String,
    pub auction_address: String,
    pub private_key: String,
    pub chain_url: String,
    pub chain_id: u64,
    pub graph_url_nft: String,
    pub graph_url_auction: String, // Add other typed environment variables here
    pub etherscan_api_key: String,
    pub mongo_atlas_username: String,
    pub mongo_atlas_password: String,
    pub alchemy_api_key: String,
    pub jwt_secret: String,
}

impl Constants {
    pub fn new() -> Self {
        dotenv::dotenv().ok(); // Load the .env file

        Constants {
            nft_address: env::var("ETH_SEPOLIA_NFT_ADDRESS").expect("NFT_ADDRESS must be set"),
            auction_address: env::var("BSC_TEST_AUCTION_ADDRESS")
                .expect("AUCTION_ADDRESS must be set"),
            private_key: env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"),
            chain_url: env::var("ETH_SEPOLIA_CHAIN_URL")
                .expect("ETH_SEPOLIA_CHAIN_URL must be set"),
            chain_id: env::var("ETH_SEPOLIA_CHAIN_ID")
                .expect("ETH_SEPOLIA_CHAIN_ID must be set")
                .parse()
                .expect("ETH_SEPOLIA_CHAIN_ID should be an integer"),
            graph_url_nft: env::var("GRAPH_URL_NFT").expect("GRAPH_URL_NFT must be set"),
            graph_url_auction: env::var("GRAPH_URL_AUCTION")
                .expect("GRAPH_URL_AUCTION must be set"),
            etherscan_api_key: env::var("ETHERSCAN_API_KEY")
                .expect("ETHERSCAN_API_KEY must be set"),
            mongo_atlas_username: env::var("MONGO_ATLAS_USERNAME")
                .expect("MONGO_ATLAS_USERNAME must be set"),
            mongo_atlas_password: env::var("MONGO_ATLAS_PASSWORD")
                .expect("MONGO_ATLAS_PASSWORD must be set"),
            alchemy_api_key: env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            // Initialize other environment variables here
        }
    }
}

pub fn with_config(
    config: Arc<Constants>,
) -> impl Filter<Extract = (Arc<Constants>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}
