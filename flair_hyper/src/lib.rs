//! A hyper based server driver

#![deny(missing_docs)]
#![deny(warnings)]

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Server};

use http::Request;

use tracing::error;
use tracing::info;

use rustserve::Route;

/// A driver for servers on Hyper
pub async fn drive(server_addr: SocketAddr, routes: Arc<Vec<Route>>) -> anyhow::Result<()> {
    let make_service = make_service_fn(move |_| {
        let routes = routes.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let routes = routes.clone();

                async move {
                    let (parts, body) = req.into_parts();

                    let bytes = to_bytes(body).await?;

                    let request = Request::from_parts(parts, &bytes[..]);

                    let res = rustserve::route_request(request, routes.clone()).await?;

                    Ok::<_, anyhow::Error>(res.map(|body| Body::from(body)))
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
