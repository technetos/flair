use crate::search::SearchController;

use rustserve::base::Reply;

#[derive(serde::Serialize)]
pub struct SearchResponse {
    pub data: String,
}

impl<'a> Reply<'a, SearchResponse> for SearchController {}
