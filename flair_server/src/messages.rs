use crate::search::SearchController;

use rustserve::Parse;
use rustserve::Reply;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LinkModel {
    pub name: String,
    pub url: String,
    pub author: String,
}

#[derive(serde::Serialize)]
pub struct SearchResponse {
    pub id: u64,
    #[serde(flatten)]
    pub model: LinkModel,
}

impl<'a> Reply<'a, SearchResponse> for SearchController {}

#[derive(serde::Deserialize)]
pub struct CreateLinkRequest {
    #[serde(flatten)]
    pub model: LinkModel,
}

impl<'a> Parse<'a, CreateLinkRequest> for SearchController {}

#[derive(serde::Serialize)]
pub struct CreateLinkResponse {
    pub id: u64,
    #[serde(flatten)]
    pub model: LinkModel,
}

impl<'a> Reply<'a, CreateLinkResponse> for SearchController {}

#[derive(serde::Deserialize)]
pub struct UpdateLinkRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
}

impl<'a> Parse<'a, UpdateLinkRequest> for SearchController {}

#[derive(serde::Serialize)]
pub struct UpdateLinkResponse {
    pub id: u64,
    #[serde(flatten)]
    pub model: LinkModel,
}

impl<'a> Reply<'a, UpdateLinkResponse> for SearchController {}
