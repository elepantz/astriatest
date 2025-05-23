// This file is @generated by prost-build.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DoBlockRequest {
    #[prost(bytes = "bytes", tag = "1")]
    pub prev_block_hash: ::prost::bytes::Bytes,
    #[prost(bytes = "bytes", repeated, tag = "2")]
    pub transactions: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for DoBlockRequest {
    const NAME: &'static str = "DoBlockRequest";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.DoBlockRequest".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.DoBlockRequest".into()
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DoBlockResponse {
    #[prost(bytes = "bytes", tag = "1")]
    pub block_hash: ::prost::bytes::Bytes,
}
impl ::prost::Name for DoBlockResponse {
    const NAME: &'static str = "DoBlockResponse";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.DoBlockResponse".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.DoBlockResponse".into()
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalizeBlockRequest {
    #[prost(bytes = "bytes", tag = "1")]
    pub block_hash: ::prost::bytes::Bytes,
}
impl ::prost::Name for FinalizeBlockRequest {
    const NAME: &'static str = "FinalizeBlockRequest";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.FinalizeBlockRequest".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.FinalizeBlockRequest".into()
    }
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct FinalizeBlockResponse {}
impl ::prost::Name for FinalizeBlockResponse {
    const NAME: &'static str = "FinalizeBlockResponse";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.FinalizeBlockResponse".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.FinalizeBlockResponse".into()
    }
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct InitStateRequest {}
impl ::prost::Name for InitStateRequest {
    const NAME: &'static str = "InitStateRequest";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.InitStateRequest".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.InitStateRequest".into()
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitStateResponse {
    #[prost(bytes = "bytes", tag = "1")]
    pub block_hash: ::prost::bytes::Bytes,
}
impl ::prost::Name for InitStateResponse {
    const NAME: &'static str = "InitStateResponse";
    const PACKAGE: &'static str = "astria.execution.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        "astria.execution.v1alpha1.InitStateResponse".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/astria.execution.v1alpha1.InitStateResponse".into()
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod execution_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ExecutionServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ExecutionServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ExecutionServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ExecutionServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            ExecutionServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn init_state(
            &mut self,
            request: impl tonic::IntoRequest<super::InitStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitStateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astria.execution.v1alpha1.ExecutionService/InitState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astria.execution.v1alpha1.ExecutionService",
                        "InitState",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn do_block(
            &mut self,
            request: impl tonic::IntoRequest<super::DoBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DoBlockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astria.execution.v1alpha1.ExecutionService/DoBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astria.execution.v1alpha1.ExecutionService",
                        "DoBlock",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn finalize_block(
            &mut self,
            request: impl tonic::IntoRequest<super::FinalizeBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FinalizeBlockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astria.execution.v1alpha1.ExecutionService/FinalizeBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astria.execution.v1alpha1.ExecutionService",
                        "FinalizeBlock",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod execution_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ExecutionServiceServer.
    #[async_trait]
    pub trait ExecutionService: std::marker::Send + std::marker::Sync + 'static {
        async fn init_state(
            self: std::sync::Arc<Self>,
            request: tonic::Request<super::InitStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitStateResponse>,
            tonic::Status,
        >;
        async fn do_block(
            self: std::sync::Arc<Self>,
            request: tonic::Request<super::DoBlockRequest>,
        ) -> std::result::Result<tonic::Response<super::DoBlockResponse>, tonic::Status>;
        async fn finalize_block(
            self: std::sync::Arc<Self>,
            request: tonic::Request<super::FinalizeBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FinalizeBlockResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct ExecutionServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ExecutionServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ExecutionServiceServer<T>
    where
        T: ExecutionService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/astria.execution.v1alpha1.ExecutionService/InitState" => {
                    #[allow(non_camel_case_types)]
                    struct InitStateSvc<T: ExecutionService>(pub Arc<T>);
                    impl<
                        T: ExecutionService,
                    > tonic::server::UnaryService<super::InitStateRequest>
                    for InitStateSvc<T> {
                        type Response = super::InitStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InitStateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ExecutionService>::init_state(inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = InitStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/astria.execution.v1alpha1.ExecutionService/DoBlock" => {
                    #[allow(non_camel_case_types)]
                    struct DoBlockSvc<T: ExecutionService>(pub Arc<T>);
                    impl<
                        T: ExecutionService,
                    > tonic::server::UnaryService<super::DoBlockRequest>
                    for DoBlockSvc<T> {
                        type Response = super::DoBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DoBlockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ExecutionService>::do_block(inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DoBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/astria.execution.v1alpha1.ExecutionService/FinalizeBlock" => {
                    #[allow(non_camel_case_types)]
                    struct FinalizeBlockSvc<T: ExecutionService>(pub Arc<T>);
                    impl<
                        T: ExecutionService,
                    > tonic::server::UnaryService<super::FinalizeBlockRequest>
                    for FinalizeBlockSvc<T> {
                        type Response = super::FinalizeBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FinalizeBlockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ExecutionService>::finalize_block(inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = FinalizeBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(empty_body());
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for ExecutionServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "astria.execution.v1alpha1.ExecutionService";
    impl<T> tonic::server::NamedService for ExecutionServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
