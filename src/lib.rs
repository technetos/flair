use rustserve::{Controller, Error, IdParam, NotFound, Parse, Reply};
use rustserve_platform::{
    ApiResponse, EntityWithId, InternalServerError, InvalidParameterError, MissingParameterError,
};

mod client;

pub use client::NameServiceClient;

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;

use futures::future::BoxFuture;

impl Error<MissingParameterError, 400> for SearchController {
    type ErrorFuture = rustserve::ErrorFuture<MissingParameterError, 400>;

    fn error(self: Arc<Self>, body: MissingParameterError) -> Self::ErrorFuture {
        rustserve::ErrorFuture::new(body, <Self as Error<MissingParameterError, 400>>::headers())
    }
}

impl Error<InvalidParameterError, 400> for SearchController {
    type ErrorFuture = rustserve::ErrorFuture<InvalidParameterError, 400>;

    fn error(self: Arc<Self>, body: InvalidParameterError) -> Self::ErrorFuture {
        rustserve::ErrorFuture::new(body, <Self as Error<InvalidParameterError, 400>>::headers())
    }
}

impl Error<InternalServerError, 500> for SearchController {
    type ErrorFuture = rustserve::ErrorFuture<InternalServerError, 500>;

    fn error(self: Arc<Self>, body: InternalServerError) -> Self::ErrorFuture {
        rustserve::ErrorFuture::new(body, <Self as Error<InternalServerError, 500>>::headers())
    }
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct HostInfo {
    pub ip: String,
    #[sqlx(try_from = "i64")]
    pub port: u16,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub hosts: Vec<EntityWithId<HostInfo>>,
}

impl Reply<ApiResponse<ServiceInfo>> for SearchController {
    type ReplyFuture = rustserve::ReplyFuture<ApiResponse<ServiceInfo>>;

    fn reply(self: Arc<Self>, body: ApiResponse<ServiceInfo>) -> Self::ReplyFuture {
        rustserve::ReplyFuture::new(body, <Self as Reply<ApiResponse<ServiceInfo>>>::headers())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct NewServiceInfo {
    pub name: String,
    #[serde(flatten)]
    pub host_info: HostInfo,
}

impl<'a> Parse<'a, NewServiceInfo> for SearchController {
    type ParseFuture = rustserve::ParseFuture<'a, NewServiceInfo>;

    fn parse(self: Arc<Self>, req: http::Request<&'a [u8]>) -> Self::ParseFuture {
        rustserve::ParseFuture::new(req)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct UpdateServiceInfo {
    pub ip: Option<String>,
    pub port: Option<u16>,
}

impl<'a> Parse<'a, EntityWithId<UpdateServiceInfo>> for SearchController {
    type ParseFuture = rustserve::ParseFuture<'a, EntityWithId<UpdateServiceInfo>>;

    fn parse(self: Arc<Self>, req: http::Request<&'a [u8]>) -> Self::ParseFuture {
        rustserve::ParseFuture::new(req)
    }
}

pub struct SearchController {
    pg_pool: Arc<PgPool>,
}

impl SearchController {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        Self { pg_pool }
    }
}

impl IdParam for SearchController {
    fn id() -> String {
        "name".into()
    }
}
impl NotFound for SearchController {}
impl Controller for SearchController {
    fn get<'a>(
        self: Arc<Self>,
        _: http::Request<&'a [u8]>,
        params: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move {
            let mut transaction = self.pg_pool.begin().await?;

            let service_name = if let Some(name) = params.get(&Self::id()) {
                if let Some(sanitized_name) = name.split_whitespace().nth(0) {
                    sanitized_name
                } else {
                    return self
                        .error(InvalidParameterError::new(&Self::id(), name))
                        .await;
                }
            } else {
                return self.error(MissingParameterError::new(&Self::id())).await;
            };

            tracing::info!("Searching for {service_name}");

            let hosts: Vec<_> = match sqlx::query_as(
                "select hosts.id, hosts.ip, hosts.port
                from services join hosts on services.id = hosts.service_id
                where services.name = $1",
            )
            .bind(service_name)
            .fetch_all(&mut transaction)
            .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row: (i64, String, i16)| {
                        Ok(EntityWithId::new(
                            u64::try_from(row.0)?,
                            HostInfo {
                                ip: row.1,
                                port: u16::try_from(row.2)?,
                            },
                        ))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?,
                Err(e) => return self.error(InternalServerError::new(format!("{e}"))).await,
            };

            transaction.commit().await?;

            self.reply(ApiResponse::new(
                "service_info",
                ServiceInfo {
                    name: service_name.into(),
                    hosts,
                },
            ))
            .await
        })
    }

    fn post<'a>(
        self: Arc<Self>,
        req: http::Request<&'a [u8]>,
        _: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move {
            let (_parts, body) = <Self as Parse<'a, NewServiceInfo>>::parse(self.clone(), req)
                .await?
                .into_parts();

            tracing::info!(
                "Creating host entry for {} @ {}:{}",
                body.name,
                body.host_info.ip,
                body.host_info.port
            );

            let mut transaction = self.pg_pool.begin().await?;

            let (service_id, service_name) = match sqlx::query_as::<_, (i64, String)>(
                "insert into services(name) values ($1) returning id, name",
            )
            .bind(body.name)
            .fetch_one(&mut transaction)
            .await
            {
                Ok((id, name)) => (u64::try_from(id)?, name),
                Err(e) => {
                    tracing::error!("{e}");
                    return Ok(self.error(InternalServerError::new(format!("{e}"))).await?);
                }
            };

            if let Err(e) = sqlx::query(
                "insert into hosts(service_id, ip, port)
                    values ($1, $2, $3)",
            )
            .bind(i64::try_from(service_id)?)
            .bind(body.host_info.ip)
            .bind(i16::try_from(body.host_info.port)?)
            .execute(&mut transaction)
            .await
            {
                tracing::error!("{e}");
                return Ok(self.error(InternalServerError::new(format!("{e}"))).await?);
            };

            let hosts: Vec<_> = match sqlx::query_as(
                "select hosts.id, hosts.ip, hosts.port
                from services join hosts on services.id = hosts.service_id
                where services.name = $1",
            )
            .bind(&service_name)
            .fetch_all(&mut transaction)
            .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row: (i64, String, i16)| {
                        Ok(EntityWithId::new(
                            u64::try_from(row.0)?,
                            HostInfo {
                                ip: row.1,
                                port: u16::try_from(row.2)?,
                            },
                        ))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?,
                Err(e) => return self.error(InternalServerError::new(format!("{e}"))).await,
            };

            transaction.commit().await?;

            self.reply(ApiResponse::new(
                "service_info",
                ServiceInfo {
                    name: service_name,
                    hosts,
                },
            ))
            .await
        })
    }

    //    fn put<'a>(
    //        self: Arc<Self>,
    //        req: http::Request<&'a [u8]>,
    //        params: HashMap<String, String>,
    //    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
    //        Box::pin(async move {
    //            let id = match u64::from_str_radix(params.get(&self.clone().id()).unwrap(), 10) {
    //                Ok(id) => id,
    //                Err(e) => {
    //                    return self
    //                        .error(BadIdError {
    //                            error: format!("Bad Id: {e}"),
    //                        })
    //                        .await
    //                }
    //            };
    //
    //            let payload: UpdateLinkRequest = match self.clone().parse(req).await {
    //                Ok(req) => req.into_body(),
    //                Err(e) => {
    //                    return self
    //                        .error(PayloadDeserializationError {
    //                            error: format!("Invalid Payload: {e}"),
    //                        })
    //                        .await
    //                }
    //            };
    //
    //            tracing::info!("Updating link: {id}");
    //
    //            let update_link_response = match Arc::make_mut(&mut self.search_service.clone())
    //                .update_link(flair_data::UpdateLinkRequest {
    //                    id,
    //                    url: payload.url,
    //                    name: payload.name,
    //                    author: payload.author,
    //                })
    //                .await
    //                .map(|r| r.into_inner())
    //            {
    //                Ok(res) => res,
    //                Err(_e) => {
    //                    return self
    //                        .error(UpdateLinkError {
    //                            error: format!(
    //                                "Service unavailable: unable to update link at this time"
    //                            ),
    //                        })
    //                        .await
    //                }
    //            };
    //
    //            self.reply(UpdateLinkResponse {
    //                id: update_link_response.id,
    //                model: LinkModel {
    //                    url: update_link_response.url,
    //                    name: update_link_response.name,
    //                    author: update_link_response.author,
    //                },
    //            })
    //            .await
    //        })
    //    }

    fn delete<'a>(
        self: Arc<Self>,
        _: http::Request<&'a [u8]>,
        _: HashMap<String, String>,
    ) -> BoxFuture<'a, anyhow::Result<http::Response<Vec<u8>>>> {
        Box::pin(async move { Ok(http::Response::builder().status(404).body(vec![])?) })
    }
}
