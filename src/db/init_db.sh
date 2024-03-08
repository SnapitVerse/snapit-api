# use snapit
# db['snapit-nft'].insertOne({ init: 1 })
# db['snapit-nft-testnet'].insertOne({ init: 1 })

mongosh <<EOF
use snapit
db.createCollection("snapit-nft-testnet")
db['snapit-nft'].createIndex({ "token_id": 1 }, { unique: true })
EOF