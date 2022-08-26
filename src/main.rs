mod proto {
    tonic::include_proto!("helloworld");
}

// use axum::{response::Response, routing::get, Router};

use tonic::{transport::Server, Request, Response, Status};

use proto::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

// #[tonic::async_trait]
// impl EvaluationService for EvaluationServiceImpl {
//     /// 渡されたトークンをもとに、利用可能な機能のコードを列挙する
//     async fn list_available_feature(
//         &self,
//         request: tonic::Request<ShowStateRequest>,
//     ) -> Result<tonic::Response<AvailableFeatures>, tonic::Status> {
//         let reply = AvailableFeatures { available: vec![] };
//         Ok(TonicResponse::new(reply))
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9002".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterService start on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
