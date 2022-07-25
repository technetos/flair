use rustserve::base::{Error, IdParam, Read, Reply};
use rustserve::Controller;

use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::DocumentNotFoundError;
use crate::errors::MissingQueryParamError;
use crate::messages::SearchResponse;

use futures::future::BoxFuture;

pub mod flair_data {
    tonic::include_proto!("search");
}
use tonic::transport::Channel;

use flair_data::search_client::SearchClient;
use flair_data::FindDocumentByNameRequest;

pub struct SearchController {
    search_service: Arc<SearchClient<Channel>>,
}

impl SearchController {
    pub fn new(search_service: Arc<SearchClient<Channel>>) -> Self {
        Self { search_service }
    }
}

impl IdParam for SearchController {
    fn id(self: Arc<Self>) -> String {
        "query".into()
    }
}

impl<'a> Read<'a, SearchResponse> for SearchController {
    fn read(
        self: Arc<Self>,
        _req: http::Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move {
            let document_name = match params.get(&self.clone().id()) {
                Some(name) => name,
                _ => return self.error(MissingQueryParamError).await,
            };

            tracing::info!("Searching for {document_name}");

            let find_document_by_name_response =
                match Arc::make_mut(&mut self.search_service.clone())
                    .find_document_by_name(FindDocumentByNameRequest {
                        name: String::from(document_name),
                    })
                    .await
                    .map(|r| r.into_inner())
                {
                    Ok(res) => res,
                    Err(_e) => return self.error(DocumentNotFoundError).await,
                };

            self.reply(SearchResponse {
                data: find_document_by_name_response.content,
            })
            .await
        })
    }
}

impl Controller for SearchController {
    fn get<'a>(
        self: Arc<Self>,
        req: http::Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        self.read(req, params)
    }
}
