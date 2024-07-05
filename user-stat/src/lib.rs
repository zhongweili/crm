pub mod abi;
mod config;
pub mod pb;
use std::{ops::Deref, pin::Pin, sync::Arc};

use futures::Stream;

use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub use config::AppConfig;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

#[allow(dead_code)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

#[tonic::async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        self.query(request.into_inner()).await
    }
    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        self.raw_query(request.into_inner()).await
    }
}

impl UserStatsService {
    pub async fn new() -> Self {
        let config = AppConfig::load().expect("failed to load config");
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("failed to connect to db");
        Self {
            inner: Arc::new(UserStatsServiceInner { config, pool }),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
