///Shorthand for `OwnedView<GetJwtPublicKeyRequestView<'static>>`.
pub type OwnedGetJwtPublicKeyRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyRequestView<'static>,
>;
///Shorthand for `OwnedView<GetJwtPublicKeyResponseView<'static>>`.
pub type OwnedGetJwtPublicKeyResponseView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyResponseView<'static>,
>;
///Shorthand for `OwnedView<RenewJwtRequestView<'static>>`.
pub type OwnedRenewJwtRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::RenewJWTRequestView<'static>,
>;
///Shorthand for `OwnedView<RenewJwtResponseView<'static>>`.
pub type OwnedRenewJwtResponseView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::RenewJWTResponseView<'static>,
>;
///Shorthand for `OwnedView<ListUsersRequestView<'static>>`.
pub type OwnedListUsersRequestView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::ListUsersRequestView<'static>,
>;
///Shorthand for `OwnedView<ListUsersResponseView<'static>>`.
pub type OwnedListUsersResponseView = ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::ListUsersResponseView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::auth::v2::GetJWTPublicKeyResponse>
for crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::auth::v2::GetJWTPublicKeyResponse>
for ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::auth::v2::RenewJWTResponse>
for crate::proto::auth::v2::__buffa::view::RenewJWTResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::auth::v2::RenewJWTResponse>
for ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::RenewJWTResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::auth::v2::ListUsersResponse>
for crate::proto::auth::v2::__buffa::view::ListUsersResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::auth::v2::ListUsersResponse>
for ::buffa::view::OwnedView<
    crate::proto::auth::v2::__buffa::view::ListUsersResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const AUTH_SERVICE_SERVICE_NAME: &str = "auth.v2.AuthService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `GetJWTPublicKey` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUTH_SERVICE_GET_JWT_PUBLIC_KEY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthService/GetJWTPublicKey",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `RenewJWT` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUTH_SERVICE_RENEW_JWT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthService/RenewJWT",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `ListUsers` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUTH_SERVICE_LIST_USERS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/auth.v2.AuthService/ListUsers",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for AuthService.
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
/// `ServiceStream<`[`StreamMessage<Req>`](::connectrpc::StreamMessage)`>`.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.7.0 and views enabled so the
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
pub trait AuthService: Send + Sync + 'static {
    /// Handle the GetJWTPublicKey RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn get_jwt_public_key<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::GetJWTPublicKeyRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::auth::v2::GetJWTPublicKeyResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the RenewJWT RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn renew_jwt<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::RenewJWTRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::auth::v2::RenewJWTResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ListUsers RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn list_users<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::auth::v2::ListUsersRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::auth::v2::ListUsersResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait AuthServiceExt: AuthService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: AuthService> AuthServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                AUTH_SERVICE_SERVICE_NAME,
                "GetJWTPublicKey",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::GetJWTPublicKeyRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get_jwt_public_key(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::proto::auth::v2::GetJWTPublicKeyResponse,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_SERVICE_GET_JWT_PUBLIC_KEY_SPEC)
            .route_view(
                AUTH_SERVICE_SERVICE_NAME,
                "RenewJWT",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::RenewJWTRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::RenewJWTRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.renew_jwt(ctx, sreq)
                                .await?
                                .encode::<crate::proto::auth::v2::RenewJWTResponse>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_SERVICE_RENEW_JWT_SPEC)
            .route_view(
                AUTH_SERVICE_SERVICE_NAME,
                "ListUsers",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::auth::v2::__buffa::view::ListUsersRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::auth::v2::ListUsersRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list_users(ctx, sreq)
                                .await?
                                .encode::<crate::proto::auth::v2::ListUsersResponse>(format)
                        }
                    })
                },
            )
            .with_spec(AUTH_SERVICE_LIST_USERS_SPEC)
    }
}
/// Monomorphic dispatcher for `AuthService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = AuthServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct AuthServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: AuthService> AuthServiceServer<T> {
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
impl<T> Clone for AuthServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: AuthService> ::connectrpc::Dispatcher for AuthServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("auth.v2.AuthService/")?;
        match method {
            "GetJWTPublicKey" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_SERVICE_GET_JWT_PUBLIC_KEY_SPEC),
                )
            }
            "RenewJWT" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_SERVICE_RENEW_JWT_SPEC),
                )
            }
            "ListUsers" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUTH_SERVICE_LIST_USERS_SPEC),
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
        let Some(method) = path.strip_prefix("auth.v2.AuthService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "GetJWTPublicKey" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::GetJWTPublicKeyRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::GetJWTPublicKeyRequest,
                    >::from_parts(&req, &body);
                    svc.get_jwt_public_key(ctx, req)
                        .await?
                        .encode::<
                            crate::proto::auth::v2::GetJWTPublicKeyResponse,
                        >(format)
                })
            }
            "RenewJWT" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::RenewJWTRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::RenewJWTRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::RenewJWTRequest,
                    >::from_parts(&req, &body);
                    svc.renew_jwt(ctx, req)
                        .await?
                        .encode::<crate::proto::auth::v2::RenewJWTResponse>(format)
                })
            }
            "ListUsers" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::auth::v2::ListUsersRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::auth::v2::__buffa::view::ListUsersRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::auth::v2::ListUsersRequest,
                    >::from_parts(&req, &body);
                    svc.list_users(ctx, req)
                        .await?
                        .encode::<crate::proto::auth::v2::ListUsersResponse>(format)
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
        let Some(method) = path.strip_prefix("auth.v2.AuthService/") else {
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
        let Some(method) = path.strip_prefix("auth.v2.AuthService/") else {
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
        let Some(method) = path.strip_prefix("auth.v2.AuthService/") else {
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
/// let client = AuthServiceClient::new(conn, config);
/// let response = client.get_jwt_public_key(request).await?;
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
/// let client = AuthServiceClient::new(http, config);
/// let response = client.get_jwt_public_key(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.get_jwt_public_key(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.get_jwt_public_key(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one `OwnedView` per
/// received message from `.message().await` — bind `msg.reborrow()` for field
/// access, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct AuthServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> AuthServiceClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::http_body::Body>::Error: ::std::fmt::Display,
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
    /// Call the GetJWTPublicKey RPC. Sends a request to /auth.v2.AuthService/GetJWTPublicKey.
    pub async fn get_jwt_public_key(
        &self,
        request: crate::proto::auth::v2::GetJWTPublicKeyRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.get_jwt_public_key_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the GetJWTPublicKey RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn get_jwt_public_key_with_options(
        &self,
        request: crate::proto::auth::v2::GetJWTPublicKeyRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::GetJWTPublicKeyResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_SERVICE_SERVICE_NAME,
                "GetJWTPublicKey",
                request,
                options,
            )
            .await
    }
    /// Call the RenewJWT RPC. Sends a request to /auth.v2.AuthService/RenewJWT.
    pub async fn renew_jwt(
        &self,
        request: crate::proto::auth::v2::RenewJWTRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::RenewJWTResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.renew_jwt_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the RenewJWT RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn renew_jwt_with_options(
        &self,
        request: crate::proto::auth::v2::RenewJWTRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::RenewJWTResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_SERVICE_SERVICE_NAME,
                "RenewJWT",
                request,
                options,
            )
            .await
    }
    /// Call the ListUsers RPC. Sends a request to /auth.v2.AuthService/ListUsers.
    pub async fn list_users(
        &self,
        request: crate::proto::auth::v2::ListUsersRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::ListUsersResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.list_users_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ListUsers RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn list_users_with_options(
        &self,
        request: crate::proto::auth::v2::ListUsersRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::auth::v2::__buffa::view::ListUsersResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUTH_SERVICE_SERVICE_NAME,
                "ListUsers",
                request,
                options,
            )
            .await
    }
}
