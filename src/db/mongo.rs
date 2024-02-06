use anyhow::Result;
use mongodb::bson::{self, doc};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub async fn init_db() -> Result<Client> {
    let mongo_uri = "mongodb://localhost:27017";
    let client_options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub async fn add_nft(client: Client, token: AddNFTInput) -> Result<()> {
    let collection = client.database("snapit").collection("snapit-nft");

    let metadata_bson = bson::to_bson(&token.metadata).map_err(|e| anyhow::Error::new(e))?; // Convert bson error to anyhow error

    let token_id_str = token.token_id.to_string();

    let new_doc = doc! {
        "tokenId": token_id_str,
        "metadata": metadata_bson,
    };

    collection
        .insert_one(new_doc, None)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    Ok(())
}

pub async fn get_nft(client: Client, token_id: u64) -> Result<Option<Value>> {
    let collection = client
        .database("snapit")
        .collection::<bson::Document>("snapit-nft");

    let filter = doc! { "tokenId": token_id.to_string() };
    let result = collection
        .find_one(filter, None)
        .await
        .map_err(|e| anyhow::Error::new(e))?; // Convert MongoDB error to anyhow error

    if let Some(document) = result {
        // Convert the BSON document to JSON
        let metadata_json = bson::to_bson(&document)
            .map_err(|e| anyhow::Error::new(e))?
            .as_document()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert BSON to Document"))?
            .clone();
        let json_value: serde_json::Value = bson::Bson::Document(metadata_json).into();
        Ok(Some(json_value))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    // Add other fields as necessary
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddNFTInput {
    pub token_id: u64,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNFT {
    // fields corresponding to your MongoDB collection
    _id: bson::oid::ObjectId,
    tokenId: String,
    metadata: serde_json::Value,
    // other fields...
}
