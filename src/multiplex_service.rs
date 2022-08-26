use axum::{body::BoxBody, response::IntoResponse};
use futures::future::BoxFuture;
use hyper::{header::CONTENT_TYPE, Body, Request, Response};
use std::{
    convert::Infallible,
    task::{Context, Poll},
};
use tower::Service;

pub struct MultiplexService<A, B> {
    rest: A,
    rest_ready: bool,
    grpc: B,
    grpc_ready: bool,
}

impl<A, B> MultiplexService<A, B> {
    pub fn new(rest: A, grpc: B) -> Self {
        Self {
            rest,
            rest_ready: false,
            grpc,
            grpc_ready: false,
        }
    }
}

impl<A, B> Clone for MultiplexService<A, B>
where
    A: Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            rest: self.rest.clone(),
            rest_ready: false,
            grpc: self.grpc.clone(),
            grpc_ready: false,
        }
    }
}

impl<A, B> Service<Request<Body>> for MultiplexService<A, B>
where
    A: Service<Request<Body>, Error = Infallible>,
    A::Response: IntoResponse,
    A::Future: Send + 'static,
    B: Service<Request<Body>, Error = Infallible>,
    B::Response: IntoResponse,
    B::Future: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            match (self.rest_ready, self.grpc_ready) {
                (true, true) => return Ok(()).into(),
                (false, _) => {
                    match self.rest.poll_ready(cx) {
                        Poll::Ready(x) => x,
                        Poll::Pending => return Poll::Pending,
                    }?;
                    self.rest_ready = true;
                }
                (_, false) => {
                    match self.grpc.poll_ready(cx) {
                        Poll::Ready(x) => x,
                        Poll::Pending => return Poll::Pending,
                    }?;
                    self.grpc_ready = true;
                }
            }
        }
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        assert!(self.rest_ready, "rest service is not ready");
        assert!(self.grpc_ready, "grpc service is not ready");

        // when calling a service it becomes not-ready so we have drive readiness again
        if is_grpc_request(&req) {
            self.grpc_ready = false;
            let future = self.grpc.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        } else {
            self.rest_ready = false;
            let future = self.rest.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        }
    }
}

fn is_grpc_request<B>(req: &hyper::Request<B>) -> bool {
    req.headers()
        .get(CONTENT_TYPE)
        .map(|content_type| content_type.as_bytes())
        .filter(|content_type| content_type.starts_with(b"application/grpc"))
        .is_some()
}
