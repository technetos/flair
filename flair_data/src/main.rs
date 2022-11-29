pub mod search {
    tonic::include_proto!("search");
}

use search::search_server::{Search, SearchServer};
use search::{CreateLinkRequest, CreateLinkResponse};
use search::{FindLinkByNameRequest, FindLinkByNameResponse};
use search::{UpdateLinkRequest, UpdateLinkResponse};

use tonic::transport::Server;
use tonic::{Request, Response, Status};

struct SearchService;

#[tonic::async_trait]
impl Search for SearchService {
    async fn find_link_by_name(
        &self,
        request: Request<FindLinkByNameRequest>,
    ) -> Result<Response<FindLinkByNameResponse>, Status> {
        let body = request.into_inner();
        Ok(Response::new(FindLinkByNameResponse {
            id: 0,
            name: body.name,
            url: "".into(),
            author: "".into(),
        }))
    }

    async fn create_link(
        &self,
        request: Request<CreateLinkRequest>,
    ) -> Result<Response<CreateLinkResponse>, Status> {
        let body = request.into_inner();
        Ok(Response::new(CreateLinkResponse {
            id: 0,
            name: body.name,
            url: body.url,
            author: body.author,
        }))
    }

    async fn update_link(
        &self,
        request: Request<UpdateLinkRequest>,
    ) -> Result<Response<UpdateLinkResponse>, Status> {
        let body = request.into_inner();
        Ok(Response::new(UpdateLinkResponse {
            id: 0,
            name: body.name.unwrap(),
            url: body.url.unwrap(),
            author: body.author.unwrap(),
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
