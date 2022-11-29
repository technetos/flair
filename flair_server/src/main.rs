//! A tiny url service

#![deny(missing_docs)]
#![deny(warnings)]

mod errors;
mod messages;
mod search;

use search::SearchController;

use rustserve::IdParam;
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
        .unwrap_or_else(|| panic!("Missing search_service_addr"))
        .value;

    let search_client = Arc::new(SearchClient::connect(search_service_addr).await?);

    let search_controller = Arc::new(SearchController::new(search_client));

    use std::str::FromStr;

    let server_addr = SocketAddr::from_str(
        &args
            .find(|arg| arg.name == "server_addr")
            .unwrap_or_else(|| panic!("Missing server_addr"))
            .value,
    )?;

    let mut routes = vec![];

    for path in generate_paths("/:version/search", search_controller.clone()) {
        routes.push(Route::filtered(
            path,
            search_controller.clone(),
            vec![search_controller.clone()],
        ))
    }

    // todo: handle these errors
    let _ = flair_hyper::drive(server_addr, Arc::new(routes)).await;

    Ok(())
}

fn generate_paths(base_path: impl Into<String>, controller: Arc<dyn IdParam>) -> Vec<String> {
    let s = base_path.into();
    let path_with_id = format!("{s}/:{}", controller.id());
    vec![s, path_with_id]
}
