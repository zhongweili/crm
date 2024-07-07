use anyhow::Result;
use config::AppConfig;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{async_trait, transport::Channel};
use user_stat::pb::user_stats_client::UserStatsClient;

mod abi;
pub mod pb;

pub mod config;

pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    metadata: MetadataClient<Channel>,
    notification: NotificationClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: tonic::Request<WelcomeRequest>,
    ) -> Result<tonic::Response<WelcomeResponse>, tonic::Status> {
        self.welcome(request.into_inner()).await
    }
    async fn recall(
        &self,
        _request: tonic::Request<RecallRequest>,
    ) -> Result<tonic::Response<RecallResponse>, tonic::Status> {
        todo!()
    }

    async fn remind(
        &self,
        _request: tonic::Request<RemindRequest>,
    ) -> Result<tonic::Response<RemindResponse>, tonic::Status> {
        todo!()
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        Ok(Self {
            config,
            user_stats,
            metadata,
            notification,
        })
    }

    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}
