# Snapit API

This project provides API for interacting with Snapit contracts.

### Prerequisites:

- Install Rust https://www.rust-lang.org/tools/install

### To run API:

```
cargo build
cargo run
```

To test API a tool like curl or an app like Postman can be used.

Sample requests:

```
curl --location 'http://localhost:3030/mint_nft' \
--header 'Content-Type: application/json' \
--data '{
    "owner_address": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
    "token_id": 5,
    "metadata": {
        "name": "nftname!"
    }
}'
```

```
curl --location 'http://localhost:3030/get_nft_metadata/1'
```

```
curl --location 'http://localhost:3030/get-owner-tokens?owner_address=0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266'
```
