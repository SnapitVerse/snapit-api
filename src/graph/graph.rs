pub fn build_graphql_query(owner_address: &str) -> String {
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
