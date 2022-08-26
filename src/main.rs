mod proto {
    tonic::include_proto!("featuremanagement.grpc");
}
mod multiplex_service;

use proto::{
    evaluation_service_server::{EvaluationService, EvaluationServiceServer},
    AvailableFeatures, Feature, ShowStateRequest,
};

use self::multiplex_service::MultiplexService;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

#[derive(Default)]
pub struct EvaluationServiceImpl {}

#[tonic::async_trait]
impl EvaluationService for EvaluationServiceImpl {
    /// 渡されたトークンをもとに、利用可能な機能のコードを列挙する
    async fn list_available_feature(
        &self,
        request: TonicRequest<ShowStateRequest>,
    ) -> Result<TonicResponse<AvailableFeatures>, Status> {
        tracing::info!("request token: {}", request.into_inner().token);
        let available = vec![Feature::Doc, Feature::Drive, Feature::Search]
            .iter()
            .map(|&f| f as i32)
            .collect();
        let features = AvailableFeatures { available };
        Ok(tonic::Response::new(features))
    }
}

async fn web_handler() -> &'static str {
    "Hello from REST!"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let rest = Router::new().route("/", get(web_handler));
    let grpc = EvaluationServiceServer::new(EvaluationServiceImpl::default());
    let service = MultiplexService::new(rest, grpc);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9002));
    tracing::info!("starting server at {}", addr);

    axum::Server::bind(&addr)
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap()
}
