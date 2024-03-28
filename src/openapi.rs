use std::sync::Arc;


use utoipa::OpenApi;

use warp::{
    hyper::Response,
    Filter, Reply,
};
use utoipa_swagger_ui::Config;

use crate::db;
use crate::chain;
use crate::handlers;
use crate::routes::{EchoRequest, EchoResponse};



// struct OpenAPIRoutes {
//     openapi_json: warp::filters::BoxedFilter<(impl warp::Reply,)>,
//     swagger_ui: warp::filters::BoxedFilter<(impl warp::Reply,)>,
// }

pub struct OpenAPIRoutes;

impl OpenAPIRoutes {

    pub fn openapi_json() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        #[derive(OpenApi)]
        #[openapi(
            paths(handlers::get_owner_tokens::get_owner_tokens_handler, handlers::mint_nft::mint_nft_handler),
            components(
                schemas(EchoRequest, EchoResponse, 
                    handlers::mint_nft::MintNFTSuccessResponse, handlers::mint_nft::MintUniqueTokenRequest, 
                    db::mongo::Metadata, db::mongo::AddNFTInput, db::mongo::MetadataAttribute,
                    chain::chain::SendTransactionResult, chain::chain::TxHashSchema, chain::chain::TransactionReceiptSchema)
            ),
            // modifiers(&SecurityAddon),
            // tags(
            //     (name = "todo", description = "Todo items management API")
            // )
        )]
        struct ApiDoc;
        warp::path!("api" / "docs" / "openapi.json").map(|| {
            let openapi: utoipa::openapi::OpenApi = ApiDoc::openapi();
            warp::reply::json(&openapi)
        })
    }

    pub fn swagger_ui() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let api_doc_config = Arc::new(Config::from("/api/docs/openapi.json"));

        warp::get()
            .and(warp::path("swagger-ui"))
            .and(warp::path::full())
            .and(warp::path::tail())
            .and(warp::any().map(move || api_doc_config.clone()))
            .and_then(serve_swagger)
    }
}

async fn serve_swagger(
    full_path: warp::path::FullPath,
    tail: warp::path::Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, warp::reject::Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(
            warp::http::Uri::from_static("/swagger-ui/"),
        )));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(warp::hyper::StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(warp::hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}




