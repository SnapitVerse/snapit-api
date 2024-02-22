use serde_json::Value;

use crate::ServerError;

pub fn graphql_owner_tokens_query(owner_address: &str) -> String {
    format!(
        r#"{{
            tokenBalances(where: {{owner: "{}"}}) {{
              token {{
                id
                metadataUri
              }}
            }}
          }}"#,
        owner_address
    )
}

pub fn graphql_token_owner_query(token_id: &str) -> String {
    format!(
        r#"{{
          tokenBalances(where: {{token_: {{id: {}}}}}) {{
            owner
          }}
        }}"#,
        token_id
    )
}

pub async fn reqwest_graphql_query(query: String, graphql_url: &str) -> Result<Value, ServerError> {
    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .post(graphql_url)
        .json(&serde_json::json!({"query": query}))
        .send()
        .await
        .map_err(|_| ServerError)?
        .json::<Value>()
        .await
        .map_err(|_| ServerError)?; // Handle HTTP request error
    Ok(res)
}
