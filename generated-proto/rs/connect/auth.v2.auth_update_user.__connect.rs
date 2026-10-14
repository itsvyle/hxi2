///Shorthand for `OwnedView<UpdateUsernameRequestView<'static>>`.
pub type OwnedUpdateUsernameRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::UpdateUsernameRequestView<'static>,
>;
///Shorthand for `OwnedView<UpdateDiscordIdRequestView<'static>>`.
pub type OwnedUpdateDiscordIdRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::UpdateDiscordIDRequestView<'static>,
>;
///Shorthand for `OwnedView<AddPasswordRequestView<'static>>`.
pub type OwnedAddPasswordRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::AddPasswordRequestView<'static>,
>;
///Shorthand for `OwnedView<RemovePasswordRequestView<'static>>`.
pub type OwnedRemovePasswordRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::RemovePasswordRequestView<'static>,
>;
/// Full service name for this service.
pub const AUTH_UPDATE_USER_SERVICE_SERVICE_NAME: &str = "auth.v2.AuthUpdateUserService";
/// Static [`Spec`](::connectrpc::Spec) for the `UpdateUsername` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const AUTH_UPDATE_USER_SERVICE_UPDATE_USERNAME_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthUpdateUserService/UpdateUsername",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `UpdateDiscordID` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const AUTH_UPDATE_USER_SERVICE_UPDATE_DISCORD_ID_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthUpdateUserService/UpdateDiscordID",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `AddPassword` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const AUTH_UPDATE_USER_SERVICE_ADD_PASSWORD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthUpdateUserService/AddPassword",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `RemovePassword` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const AUTH_UPDATE_USER_SERVICE_REMOVE_PASSWORD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthUpdateUserService/RemovePassword",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for AuthUpdateUserService.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// [`InboundStream<Req>`](::connectrpc::InboundStream) — a
/// `ServiceStream` of [`StreamMessage`](::connectrpc::StreamMessage)s.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.9.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity)]
pub trait AuthUpdateUserService: Send + Sync + 'static {
    /// Handle the UpdateUsername RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn update_username<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::UpdateUsernameRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// / For now, do not enable this, as this could allow chaos
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn update_discord_id<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::UpdateDiscordIDRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the AddPassword RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn add_password<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::AddPasswordRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the RemovePassword RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn remove_password<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::RemovePasswordRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
/// Prefer [`Router::add_service`](::connectrpc::Router::add_service) for
/// top-down registration; `register` remains available for compatibility
/// and cases where the service-first call shape is more convenient.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait AuthUpdateUserServiceExt: AuthUpdateUserService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: AuthUpdateUserService> AuthUpdateUserServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                AUTH_UPDATE_USER_SERVICE_SERVICE_NAME,
                "UpdateUsername",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::UpdateUsernameRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::UpdateUsernameRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.update_username(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_UPDATE_USER_SERVICE_UPDATE_USERNAME_SPEC)
            .route_view(
                AUTH_UPDATE_USER_SERVICE_SERVICE_NAME,
                "UpdateDiscordID",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::UpdateDiscordIDRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::UpdateDiscordIDRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.update_discord_id(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_UPDATE_USER_SERVICE_UPDATE_DISCORD_ID_SPEC)
            .route_view(
                AUTH_UPDATE_USER_SERVICE_SERVICE_NAME,
                "AddPassword",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::AddPasswordRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::AddPasswordRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add_password(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_UPDATE_USER_SERVICE_ADD_PASSWORD_SPEC)
            .route_view(
                AUTH_UPDATE_USER_SERVICE_SERVICE_NAME,
                "RemovePassword",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::RemovePasswordRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::RemovePasswordRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.remove_password(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_UPDATE_USER_SERVICE_REMOVE_PASSWORD_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct AuthUpdateUserServiceRegisterMarker;
impl<
    S: AuthUpdateUserService,
> ::connectrpc::ServiceRegister<AuthUpdateUserServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as AuthUpdateUserServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `AuthUpdateUserService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = AuthUpdateUserServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct AuthUpdateUserServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: AuthUpdateUserService> AuthUpdateUserServiceServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for AuthUpdateUserServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: AuthUpdateUserService> ::connectrpc::Dispatcher
for AuthUpdateUserServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("auth.v2.AuthUpdateUserService/")?;
        match method {
            "UpdateUsername" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_UPDATE_USER_SERVICE_UPDATE_USERNAME_SPEC),
                )
            }
            "UpdateDiscordID" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_UPDATE_USER_SERVICE_UPDATE_DISCORD_ID_SPEC),
                )
            }
            "AddPassword" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_UPDATE_USER_SERVICE_ADD_PASSWORD_SPEC),
                )
            }
            "RemovePassword" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_UPDATE_USER_SERVICE_REMOVE_PASSWORD_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("auth.v2.AuthUpdateUserService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "UpdateUsername" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::UpdateUsernameRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::UpdateUsernameRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::UpdateUsernameRequest,
                    >::from_parts(&req, &body);
                    svc.update_username(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            "UpdateDiscordID" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::UpdateDiscordIDRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::UpdateDiscordIDRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::UpdateDiscordIDRequest,
                    >::from_parts(&req, &body);
                    svc.update_discord_id(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            "AddPassword" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::AddPasswordRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::AddPasswordRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::AddPasswordRequest,
                    >::from_parts(&req, &body);
                    svc.add_password(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            "RemovePassword" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::RemovePasswordRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::RemovePasswordRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::RemovePasswordRequest,
                    >::from_parts(&req, &body);
                    svc.remove_password(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("auth.v2.AuthUpdateUserService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("auth.v2.AuthUpdateUserService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("auth.v2.AuthUpdateUserService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = AuthUpdateUserServiceClient::new(conn, config);
/// let response = client.update_username(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = AuthUpdateUserServiceClient::new(http, config);
/// let response = client.update_username(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.update_username(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.update_username(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct AuthUpdateUserServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> AuthUpdateUserServiceClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::connectrpc::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the UpdateUsername RPC. Sends a request to /auth.v2.AuthUpdateUserService/UpdateUsername.
    pub async fn update_username(
        &self,
        request: crate::proto::auth::v2::UpdateUsernameRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.update_username_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the UpdateUsername RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn update_username_with_options(
        &self,
        request: crate::proto::auth::v2::UpdateUsernameRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_UPDATE_USER_SERVICE_UPDATE_USERNAME_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the UpdateDiscordID RPC. Sends a request to /auth.v2.AuthUpdateUserService/UpdateDiscordID.
    pub async fn update_discord_id(
        &self,
        request: crate::proto::auth::v2::UpdateDiscordIDRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.update_discord_id_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the UpdateDiscordID RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn update_discord_id_with_options(
        &self,
        request: crate::proto::auth::v2::UpdateDiscordIDRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_UPDATE_USER_SERVICE_UPDATE_DISCORD_ID_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the AddPassword RPC. Sends a request to /auth.v2.AuthUpdateUserService/AddPassword.
    pub async fn add_password(
        &self,
        request: crate::proto::auth::v2::AddPasswordRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.add_password_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the AddPassword RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn add_password_with_options(
        &self,
        request: crate::proto::auth::v2::AddPasswordRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_UPDATE_USER_SERVICE_ADD_PASSWORD_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the RemovePassword RPC. Sends a request to /auth.v2.AuthUpdateUserService/RemovePassword.
    pub async fn remove_password(
        &self,
        request: crate::proto::auth::v2::RemovePasswordRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.remove_password_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the RemovePassword RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn remove_password_with_options(
        &self,
        request: crate::proto::auth::v2::RemovePasswordRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_UPDATE_USER_SERVICE_REMOVE_PASSWORD_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
}
