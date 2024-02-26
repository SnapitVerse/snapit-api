use std::env;

pub struct Constants {
    pub nft_address: String,
    pub auction_address: String,
    pub private_key: String,
    pub chain_url: String,
    pub chain_id: u64,
    pub graph_url_token: String,
    pub graph_url_auction: String, // Add other typed environment variables here
}

impl Constants {
    pub fn new() -> Self {
        dotenv::dotenv().ok(); // Load the .env file

        Constants {
            nft_address: env::var("NFT_ADDRESS").expect("DATABASE_URL must be set"),
            auction_address: env::var("AUCTION_ADDRESS").expect("DATABASE_URL must be set"),
            private_key: env::var("PRIVATE_KEY").expect("API_KEY must be set"),
            chain_url: env::var("CHAIN_URL").expect("API_KEY must be set"),
            chain_id: env::var("CHAIN_ID")
                .expect("API_KEY must be set")
                .parse()
                .expect("MY_INTEGER should be an integer"),
            graph_url_token: env::var("GRAPH_URL_TOKEN").expect("API_KEY must be set"),
            graph_url_auction: env::var("GRAPH_URL_AUCTION").expect("API_KEY must be set"),
            // Initialize other environment variables here
        }
    }
}
