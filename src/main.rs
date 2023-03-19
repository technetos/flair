//! A tiny name service

#![deny(missing_docs)]
#![deny(warnings)]

use flair::SearchController;

use rustserve::IdParam;
use rustserve::Route;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;

use rustserve::{Controller, Filter};
use rustserve_platform::{default_filters, runtime};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = args().into_iter();

    let pg_pool = Arc::new(
        PgPoolOptions::new()
            .connect(&std::env::var("DATABASE_URL")?)
            .await?,
    );

    let search_controller = Arc::new(SearchController::new(pg_pool.clone()));

    use std::str::FromStr;

    let server_addr = SocketAddr::from_str(
        &args
            .find(|arg| arg.name == "server_addr")
            .unwrap_or_else(|| panic!("Missing server_addr"))
            .value,
    )?;

    let mut routes = vec![];

    add_routes(
        &mut routes,
        generate_routes(
            "/:version/services",
            search_controller,
            default_filters::<SearchController>(),
        ),
    );

    Ok(runtime::drive(server_addr, Arc::new(routes), true, "flair_mtls").await?)
}

fn generate_routes<T: IdParam + Controller + 'static>(
    base_path: impl Into<String>,
    controller: Arc<T>,
    filters: Vec<Arc<dyn Filter>>,
) -> Vec<Route> {
    let id = T::id();
    let s = base_path.into();
    let with_id = format!("{s}/:{id}");

    vec![
        Route::filtered(s, controller.clone(), filters.clone()),
        Route::filtered(with_id, controller, filters),
    ]
}

fn add_routes(routes: &mut Vec<Route>, new_routes: Vec<Route>) {
    for route in &new_routes {
        tracing::info!("{}", route.full_path());
    }

    routes.extend(new_routes);
}

struct Arg {
    pub name: String,
    pub value: String,
}

fn args_impl(input: impl Iterator<Item = String>) -> Vec<Arg> {
    input
        .skip(1)
        .map(|arg| {
            let pos = arg.chars().position(|v| v == '=');
            match arg {
                s if s.starts_with("-") && pos.is_some() => {
                    let pos = pos.unwrap();
                    let name = String::from(&s[1..pos]);
                    let value = String::from(&s[pos + 1..]);

                    Arg { name, value }
                }
                _ => panic!("Invalid commandline arguments"),
            }
        })
        .collect()
}

fn args() -> Vec<Arg> {
    args_impl(std::env::args())
}
