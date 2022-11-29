use rustserve::{
    Controller, Error, Filter, IdParam, Parse, Reply, RequestFilterOutcome, ResponseFilterOutcome,
};

use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::CreateLinkError;
use crate::errors::LinkNotFoundError;
use crate::errors::MissingQueryParamError;
use crate::errors::PayloadDeserializationError;
use crate::messages::CreateLinkResponse;
use crate::messages::LinkModel;
use crate::messages::SearchResponse;

use futures::future::BoxFuture;

pub mod flair_data {
    tonic::include_proto!("search");
}
use tonic::transport::Channel;

use flair_data::search_client::SearchClient;

pub struct SearchController {
    search_service: Arc<SearchClient<Channel>>,
}

impl SearchController {
    pub fn new(search_service: Arc<SearchClient<Channel>>) -> Self {
        Self { search_service }
    }
}

impl IdParam for SearchController {}

impl Filter for SearchController {
    fn filter_request<'a>(
        self: Arc<Self>,
        req: http::Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<RequestFilterOutcome<'a>>> {
        Box::pin(async move {
            if req.method() == "POST" && params.contains_key(&self.id()) {
                return Ok(RequestFilterOutcome::Fail(
                    http::Response::builder().status(404).body(vec![])?,
                ));
            }
            Ok(RequestFilterOutcome::Pass(req, params))
        })
    }

    fn filter_response<'a>(
        self: Arc<Self>,
        res: http::Response<Vec<u8>>,
    ) -> BoxFuture<'a, anyhow::Result<ResponseFilterOutcome>> {
        Box::pin(async move { Ok(ResponseFilterOutcome::Pass(res)) })
    }
}

impl Controller for SearchController {
    fn get<'a>(
        self: Arc<Self>,
        _: http::Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move {
            let document_name = match params.get(&self.clone().id()) {
                Some(name) => name,
                _ => return self.error(MissingQueryParamError).await,
            };

            tracing::info!("Searching for {document_name}");

            let find_link_by_name_response = match Arc::make_mut(&mut self.search_service.clone())
                .find_link_by_name(flair_data::FindLinkByNameRequest {
                    name: String::from(document_name),
                })
                .await
                .map(|r| r.into_inner())
            {
                Ok(res) => res,
                Err(_e) => return self.error(LinkNotFoundError).await,
            };

            self.reply(SearchResponse {
                id: find_link_by_name_response.id,
                model: LinkModel {
                    name: find_link_by_name_response.name,
                    url: find_link_by_name_response.url,
                    author: find_link_by_name_response.author,
                },
            })
            .await
        })
    }

    fn post<'a>(
        self: Arc<Self>,
        req: http::Request<&'a [u8]>,
        _: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move {
            let (_, payload) = match self.clone().parse(req).await {
                Ok(req) => req.into_parts(),
                Err(e) => {
                    return self
                        .error(PayloadDeserializationError {
                            error: format!("Invalid Payload: {e}"),
                        })
                        .await
                }
            };

            tracing::info!(
                "Creating link: {} for url: {}",
                payload.model.name,
                payload.model.url
            );

            let create_link_response = match Arc::make_mut(&mut self.search_service.clone())
                .create_link(flair_data::CreateLinkRequest {
                    url: payload.model.url,
                    name: payload.model.name,
                    author: payload.model.author,
                })
                .await
                .map(|r| r.into_inner())
            {
                Ok(res) => res,
                Err(_e) => {
                    return self
                        .error(CreateLinkError {
                            error: format!(
                                "Service unavailable: unable to create link at this time"
                            ),
                        })
                        .await
                }
            };

            self.reply(CreateLinkResponse {
                id: create_link_response.id,
                model: LinkModel {
                    name: create_link_response.name,
                    url: create_link_response.url,
                    author: create_link_response.author,
                },
            })
            .await
        })
    }
}
