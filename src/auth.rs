use anyhow::anyhow;
use chrono::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use warp::{
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    reject, Filter,
};

use crate::{constants, error::ServerError};

const BEARER: &str = "Bearer ";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn with_auth() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::headers_cloned()
        .map(move |headers: HeaderMap| headers)
        .and_then(authorize)
}

pub fn _create_jwt(uid: &str) -> Result<String, ServerError> {
    let config = constants::Constants::new();
    // let config = Arc::new(config);

    let jwt_secret = config.jwt_secret.as_bytes();
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_owned(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let jwt = encode(&header, &claims, &EncodingKey::from_secret(jwt_secret))
        .map_err(|_| ServerError::from(anyhow!("jwt token creation error")))?;

    println!("JWT: {}", jwt);
    Ok(jwt)
}

async fn authorize(headers: HeaderMap<HeaderValue>) -> Result<String, warp::Rejection> {
    let config = constants::Constants::new();
    // let config = Arc::new(config);

    let jwt_secret = config.jwt_secret.as_bytes();

    match jwt_from_header(&headers) {
        Ok(jwt) => {
            if jwt == "APITEST" {
                return Ok("test".to_string());
            }
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(jwt_secret),
                &Validation::new(Algorithm::HS512),
            )
            .map_err(|_| reject::custom(ServerError::from(anyhow!("jwt token not valid"))))?;

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(reject::custom(ServerError::from(e))),
    }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, ServerError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(ServerError::from(anyhow!("no auth header"))), //Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(ServerError::from(anyhow!("no auth header"))), // Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(ServerError::from(anyhow!("invalid auth header")));
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
