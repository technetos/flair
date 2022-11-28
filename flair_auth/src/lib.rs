use http::{Request, Response};
use std::sync::Arc;
use std::collections::HashMap;
use futures::future::BoxFuture;
use rustserve::{Filter, RequestFilterOutcome, ResponseFilterOutcome};

pub struct LinkAuthor;

impl Filter for LinkAuthor {
    fn filter_request<'a>(
        self: Arc<Self>,
        req: Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<RequestFilterOutcome<'a>>> {
        Box::pin(async move {
            Ok(RequestFilterOutcome::Pass(req, params))
        })
    }

    fn filter_response<'a>(
        self: Arc<Self>,
        res: Response<Vec<u8>>,
    ) -> BoxFuture<'a, anyhow::Result<ResponseFilterOutcome>> {
        Box::pin(async move {
            Ok(ResponseFilterOutcome::Pass(res))
        })
    }
}
