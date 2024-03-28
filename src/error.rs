use ethers::providers::{Middleware, ProviderError};
use warp::{http::StatusCode, Rejection, Reply};

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(server_error) = err.find::<ServerError>() {
        // Here we handle the custom ServerError
        code = StatusCode::UNAUTHORIZED; // Or any other appropriate status code
        message = server_error._reason.as_str();
    } else if err.find::<warp::reject::InvalidQuery>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Query";
    } else {
        // Log the unhandled rejection
        eprintln!("Unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&{
        let mut map = std::collections::HashMap::new();
        map.insert("message", message);
        map
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(Debug)]
pub struct ServerError {
    _reason: String,
}

impl From<anyhow::Error> for ServerError {
    fn from(err: anyhow::Error) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl<M> From<ethers::contract::ContractError<M>> for ServerError
where
    M: Middleware,
{
    fn from(err: ethers::contract::ContractError<M>) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<ProviderError> for ServerError {
    fn from(err: ProviderError) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl From<ethers::contract::AbiError> for ServerError {
    fn from(err: ethers::contract::AbiError) -> ServerError {
        ServerError {
            _reason: err.to_string(),
        }
    }
}

impl warp::reject::Reject for ServerError {}
