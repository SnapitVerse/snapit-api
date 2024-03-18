// use ethers::{types::Bytes, utils::hex};
// use serde::Serialize;

// pub fn object_to_data_bytes<T: Serialize>(value: T) -> Bytes {
//     let metadata_json = serde_json::to_string(&value).expect("Failed to serialize");
//     // let metadata_json = serde_json::to_string(&value).unwrap();
//     let metadata_hex = hex::encode(metadata_json);

//     Bytes::from(hex::decode(metadata_hex).expect("Invalid hex"))
// }
