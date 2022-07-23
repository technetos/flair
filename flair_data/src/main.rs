pub mod search {
    tonic::include_proto!("search");
}

use search::search_server::{Search, SearchServer};
use search::{FindDocumentByNameRequest, FindDocumentByNameResponse};

use tonic::transport::Server;
use tonic::{Request, Response, Status};

struct SearchService;

#[tonic::async_trait]
impl Search for SearchService {
    async fn find_document_by_name(
        &self,
        request: Request<FindDocumentByNameRequest>,
    ) -> Result<Response<FindDocumentByNameResponse>, Status> {
        dbg!(request);
        Ok(Response::new(FindDocumentByNameResponse {
            id: 0,
            content: String::from("test content"),
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = flair_args::args().into_iter();

    let server_addr = args.find(|arg| arg.name == "server_addr").unwrap().value;

    let search_service = SearchService {};

    let service = SearchServer::new(search_service);

    use std::str::FromStr;

    Server::builder()
        .add_service(service)
        .serve(std::net::SocketAddr::from_str(&server_addr)?)
        .await?;

    Ok(())
}
