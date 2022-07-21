use crate::search::SearchController;

use rustserve::base::Error;

use http::Response;

use std::sync::Arc;

use futures::future::BoxFuture;

#[derive(serde::Serialize)]
pub struct InternalServerError;

macro_rules! internal_server_error {
    ($name:ident) => {
        impl<'a> Error<'a, InternalServerError, 500> for $name {
            fn error(
                self: Arc<Self>,
                _body: InternalServerError,
            ) -> BoxFuture<'a, anyhow::Result<Response<Vec<u8>>>> {
                Box::pin(async move { Ok(Response::builder().status(500).body(vec![])?) })
            }
        }
    };
}

internal_server_error!(SearchController);

#[derive(serde::Serialize)]
pub struct DocumentNotFoundError;

impl<'a> Error<'a, DocumentNotFoundError, 404> for SearchController {}
