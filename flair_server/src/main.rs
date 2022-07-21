#![deny(warnings)]

mod errors;
mod messages;
mod search;

use search::SearchController;

use rustserve::base::IdParam;
use rustserve::Route;

use http::{Request, Response};
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Server};

use tracing::{error, info};

use search::flair_data::search_client::SearchClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = flair_args::args();

    let mut iter = args.into_iter();

    use std::str::FromStr;

    let search_service_addr = iter
        .find(|arg| arg.name == "search_service_addr")
        .unwrap()
        .value;

    let search_client = Arc::new(SearchClient::connect(search_service_addr).await?);

    let search_controller = Arc::new(SearchController::new(search_client));

    let server_addr = std::net::SocketAddr::from_str(&iter
        .find(|arg| arg.name == "server_addr")
        .unwrap()
        .value)?;

    let routes = Arc::new(vec![Route::new(
        format!("/:version/search/:{}", search_controller.clone().id()),
        search_controller.clone(),
    )]);

    let make_service = make_service_fn(move |_| {
        let routes = routes.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let routes = routes.clone();

                async move {
                    let (parts, body) = req.into_parts();

                    let bytes = match hyper::body::to_bytes(body).await {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            tracing::warn!("{e}");
                            return Ok(Response::builder()
                                .status(500)
                                .body(Body::empty())
                                .unwrap());
                        }
                    };

                    let request = Request::from_parts(parts, &bytes[..]);

                    let res = rustserve::route_request(request, routes.clone())
                        .await
                        .unwrap();

                    Ok::<_, Error>(res.map(|body| Body::from(body)))
                }
            }))
        }
    });

    let server = Server::bind(&server_addr).serve(make_service);

    info!("Listening on http://{server_addr}");

    if let Err(e) = server.await {
        error!("{e}");
    }

    Ok(())
}
