#![deny(warnings)]

mod errors;
mod messages;
mod search;

use search::SearchController;

use rustserve::base::IdParam;
use rustserve::Route;

use std::net::SocketAddr;
use std::sync::Arc;

use search::flair_data::search_client::SearchClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = flair_args::args().into_iter();

    let search_service_addr = args
        .find(|arg| arg.name == "search_service_addr")
        .unwrap()
        .value;

    let search_client = Arc::new(SearchClient::connect(search_service_addr).await?);

    let search_controller = Arc::new(SearchController::new(search_client));

    use std::str::FromStr;

    let server_addr =
        SocketAddr::from_str(&args.find(|arg| arg.name == "server_addr").unwrap().value)?;

    let routes = Arc::new(vec![Route::new(
        format!("/:version/search/:{}", search_controller.clone().id()),
        search_controller.clone(),
    )]);

    // todo: handle these errors
    let _ = flair_hyper::drive(server_addr, routes).await;

    Ok(())
}
