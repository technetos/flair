use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;

use rustserve_platform::client::make_and_send_request;
use rustserve_platform::client::CertificatePath;
use rustserve_platform::ApiResponse;

use rustserve::ServiceRequest;

use crate::HostInfo;
use crate::NewServiceInfo;
use crate::ServiceInfo;

impl<'a> ServiceRequest<'a, (), ApiResponse<ServiceInfo>> for NameServiceClient {
    fn method() -> http::Method {
        http::Method::GET
    }

    type ResponseFuture = rustserve::ParseResponseFuture<ApiResponse<ServiceInfo>>;

    fn parse_response(self: Arc<Self>, res: http::Response<Vec<u8>>) -> Self::ResponseFuture {
        rustserve::ParseResponseFuture::new(res)
    }
}

impl<'a> ServiceRequest<'a, NewServiceInfo, ApiResponse<ServiceInfo>> for NameServiceClient {
    fn method() -> http::Method {
        http::Method::POST
    }

    type ResponseFuture = rustserve::ParseResponseFuture<ApiResponse<ServiceInfo>>;

    fn parse_response(self: Arc<Self>, res: http::Response<Vec<u8>>) -> Self::ResponseFuture {
        rustserve::ParseResponseFuture::new(res)
    }
}

impl<'a, Req, Res> CertificatePath<'a, Req, Res> for NameServiceClient
where
    Req: serde::Serialize + Send + 'a,
    Res: for<'de> serde::Deserialize<'de> + Send + 'a,
{
    fn cert_path(self: Arc<Self>) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let service_name = <Self as rustserve::ServiceInfo<'a, Req, Res>>::name();

            let cert_root_path = std::env::var("CERTIFICATE_ROOT").unwrap_or(".".into());
            Ok(format!("{cert_root_path}/{service_name}/rsa/end.chain"))
        })
    }
}

impl<'a, Req, Res> rustserve::ServiceInfo<'a, Req, Res> for NameServiceClient {
    fn name() -> &'static str {
        "flair_mtls"
    }

    fn addr(self: Arc<Self>) -> BoxFuture<'a, anyhow::Result<String>> {
        // Hardcoded for now
        Box::pin(async move { Ok(String::from("localhost:3053")) })
    }

    fn additional_headers(
        self: Arc<Self>,
    ) -> BoxFuture<'a, anyhow::Result<HashMap<String, String>>> {
        Box::pin(async move { Ok(HashMap::from([("host".into(), "localhost".into())])) })
    }
}

/// Client for talking to Data Service.
pub struct NameServiceClient;

impl NameServiceClient {
    /// Create a new instance of the NameServiceClient
    pub fn new() -> Self {
        Self
    }

    /// Create a new service entry with a single host
    pub async fn create_service_entry(
        self: Arc<Self>,
        name: impl Into<String>,
        ip: impl Into<String>,
        port: u16,
    ) -> anyhow::Result<http::Response<ApiResponse<ServiceInfo>>> {
        let name = name.into();
        let ip = ip.into();
        let path = format!("/1/services");

        make_and_send_request(
            self.clone(),
            &path,
            NewServiceInfo {
                name,
                host_info: HostInfo { ip, port },
            },
        )
        .await
    }

    /// Lookup a service host by service name
    pub async fn lookup(
        self: Arc<Self>,
        service_name: impl Into<String>,
    ) -> anyhow::Result<http::Response<ApiResponse<ServiceInfo>>> {
        let service_name = service_name.into();
        let path = format!("/1/services/{service_name}");

        make_and_send_request(self.clone(), &path, ()).await
    }
}
