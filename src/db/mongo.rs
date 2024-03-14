use anyhow::Result;
use futures::TryStreamExt;
use mongodb::bson::{self, doc};
use mongodb::options::{FindOneOptions, ServerApi, ServerApiVersion};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use warp::filters::body::json;

use crate::constants::Constants;

const COLLECTION_NAME: &str = "snapit-nft-testnet";
const SETTINGS_COLLECTION_NAME: &str = "settings";

pub async fn init_db(config: Arc<Constants>) -> Result<Client> {
    let mongo_uri = format!("mongodb+srv://{}:{}@test-snapit-api.zowevot.mongodb.net/?retryWrites=true&w=majority&appName=test-snapit-api", config.mongo_atlas_username, config.mongo_atlas_password);

    let mut client_options = ClientOptions::parse(mongo_uri).await?;

    // Set the server_api field of the client_options object to set the version of the Stable API on the client
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;
    // Ping the server to see if you can connect to the cluster
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    Ok(client)
}

pub async fn add_nft(client: Arc<Client>, token: AddNFTInput) -> Result<()> {
    let collection = client.database("snapit").collection(COLLECTION_NAME);

    let metadata_bson = bson::to_bson(&token.metadata).map_err(|e| anyhow::Error::new(e))?; // Convert bson error to anyhow error

    let token_id_str = token.token_id.to_string();

    let new_doc = doc! {
        "token_id": token_id_str,
        "metadata": metadata_bson,
    };

    collection
        .insert_one(new_doc, None)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    Ok(())
}

pub async fn contract_metadata(client: Arc<Client>) -> Result<Option<ContractMetadata>> {
    let collection = client
        .database("snapit")
        .collection::<bson::Document>(SETTINGS_COLLECTION_NAME);

    let filter = doc! { "type": "contract-metadata", "version": "0.0.1" };
    let find_option: FindOneOptions = FindOneOptions::builder()
        .projection(doc! { "_id": 0, "type": 0, "version": 0 })
        .build();
    let result = collection
        .find_one(filter, find_option)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    if let Some(document) = result {
        // Convert the BSON document to JSON
        let metadata_json = bson::to_bson(&document)
            .map_err(|e| anyhow::Error::new(e))?
            .as_document()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert BSON to Document"))?
            .clone();
        let json_value: SettingsContractMetadata = bson::from_document(metadata_json)?;
        Ok(Some(json_value.metadata))
    } else {
        Ok(None)
    }
}

pub async fn find_one_nft(client: Arc<Client>, token_id: u64) -> Result<Option<DBNFTWithoutId>> {
    let collection = client
        .database("snapit")
        .collection::<bson::Document>(COLLECTION_NAME);

    let filter = doc! { "token_id": token_id.to_string() };
    let find_option: FindOneOptions = FindOneOptions::builder()
        .projection(doc! { "_id": 0 })
        .build();
    let result = collection
        .find_one(filter, find_option)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    if let Some(document) = result {
        // Convert the BSON document to JSON
        let metadata_json = bson::to_bson(&document)
            .map_err(|e| anyhow::Error::new(e))?
            .as_document()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert BSON to Document"))?
            .clone();
        let json_value: DBNFTWithoutId = bson::from_document(metadata_json)?;
        Ok(Some(json_value))
    } else {
        Ok(None)
    }
}

pub async fn find_nfts(client: Arc<Client>, token_ids: Vec<u64>) -> Result<Vec<DBNFTWithoutId>> {
    let collection = client
        .database("snapit")
        .collection::<bson::Document>(COLLECTION_NAME);

    // Convert token_ids to strings and prepare for $in query
    let token_ids_str: Vec<String> = token_ids.iter().map(|id| id.to_string()).collect();

    let filter = doc! { "token_id": { "$in": token_ids_str } };

    let mut cursor = collection
        .find(filter, None)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    let mut results = Vec::new();
    while let Some(document) = cursor.try_next().await.map_err(|e| anyhow::Error::new(e))? {
        // Convert each BSON document to JSON
        let metadata_json = bson::to_bson(&document)
            .map_err(|e| anyhow::Error::new(e))?
            .as_document()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert BSON to Document"))?
            .clone();
        let json_value: DBNFTWithoutId = bson::from_document(metadata_json)?;
        results.push(json_value);
    }

    Ok(results)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: String,
    pub collaborators: Vec<String>, // Add other fields as necessary
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsContractMetadata {
    pub metadata: ContractMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_url: String,
    pub attributes: Vec<MetadataAttribute>, // Add other fields as necessary
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataAttribute {
    trait_type: String,
    display_type: Option<String>,
    value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddNFTInput {
    pub token_id: u64,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNFT {
    // fields corresponding to your MongoDB collection
    _id: bson::oid::ObjectId,
    token_id: String,
    metadata: Metadata,
    // other fields...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNFTWithoutId {
    // fields corresponding to your MongoDB collection
    pub token_id: String,
    pub metadata: Metadata,
    // other fields...
}
