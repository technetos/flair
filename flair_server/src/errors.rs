use crate::search::SearchController;

use rustserve::Error;

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
pub struct LinkNotFoundError;

impl<'a> Error<'a, LinkNotFoundError, 404> for SearchController {}

#[derive(serde::Serialize)]
pub struct MissingQueryParamError;

impl<'a> Error<'a, MissingQueryParamError, 400> for SearchController {}

#[derive(serde::Serialize)]
pub struct PayloadDeserializationError {
    pub error: String,
}

impl<'a> Error<'a, PayloadDeserializationError, 400> for SearchController {}

#[derive(serde::Serialize)]
pub struct CreateLinkError {
    pub error: String,
}

impl<'a> Error<'a, CreateLinkError, 503> for SearchController {}

#[derive(serde::Serialize)]
pub struct UpdateLinkError {
    pub error: String,
}

impl<'a> Error<'a, UpdateLinkError, 503> for SearchController {}

#[derive(serde::Serialize)]
pub struct BadIdError {
    pub error: String,
}

impl<'a> Error<'a, BadIdError, 400> for SearchController {}
