use anyhow::Result;
use axum::{routing::get, Router};
use hxi2_proto::connectrpc::Router as ConnectRouter;
use std::sync::Arc;

use hxi2_proto::connect::auth::v1::{GreetService, GreetServiceExt};
use hxi2_proto::connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use hxi2_proto::proto::auth::v1::{GreetRequest, GreetResponse};

struct MyGreetService;

#[allow(refining_impl_trait)]
impl GreetService for MyGreetService {
    async fn greet(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, GreetRequest>,
    ) -> ServiceResult<GreetResponse> {
        // `request` derefs to the view — string fields are borrowed `&str`
        // directly from the request buffer (zero-copy). The borrow lives for
        // the duration of the call; use `request.to_owned_message()` for
        // anything that must outlive it (e.g. `tokio::spawn`).
        Response::ok(GreetResponse {
            message: format!("Hello, {}!", request.name),
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(MyGreetService);
    let connect = service.register(ConnectRouter::new());

    // Plain HTTP liveness probe for `kubectl`'s httpGet style. For the
    // standard gRPC Health protocol (grpc_health_probe, kubelet `grpc:`
    // probes), mount `connectrpc_health::HealthService` on the Connect
    // router instead — see docs/guide.md#health-checking.
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
