use std::{
    net::SocketAddr,
    sync::Arc,
};

use astria_core::{
    self,
    generated::{
        astria::sequencerblock::v1::{
            sequencer_service_server::{
                SequencerService,
                SequencerServiceServer,
            },
            FilteredSequencerBlock as RawFilteredSequencerBlock,
            GetFilteredSequencerBlockRequest,
            GetPendingNonceRequest,
            GetPendingNonceResponse,
            GetSequencerBlockRequest,
            SequencerBlock as RawSequencerBlock,
        },
        sequencerblock::v1::{
            GetUpgradesInfoRequest,
            GetUpgradesInfoResponse,
            GetValidatorNameRequest,
            GetValidatorNameResponse,
        },
    },
};
use astria_eyre::eyre::{
    self,
    WrapErr as _,
};
use astria_grpc_mock::{
    matcher::message_type,
    response::constant_response,
    Mock,
    MockServer,
};
use tokio::task::JoinHandle;
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

// NOTE: the actual full path name is
// /astria.sequencerblock.v1.SequencerService/GetPendingNonce
const GET_PENDING_NONCE_GRPC_NAME: &str = "get_pending_nonce";

pub struct MockGrpcSequencer {
    _server: JoinHandle<eyre::Result<()>>,
    pub(crate) mock_server: MockServer,
    pub(crate) local_addr: SocketAddr,
}

impl MockGrpcSequencer {
    pub(crate) async fn spawn() -> Self {
        use tokio_stream::wrappers::TcpListenerStream;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let mock_server = MockServer::new();

        let server = {
            let sequencer_service = SequencerServiceImpl(mock_server.clone());
            tokio::spawn(async move {
                Server::builder()
                    .add_service(SequencerServiceServer::new(sequencer_service))
                    .serve_with_incoming(TcpListenerStream::new(listener))
                    .await
                    .wrap_err("gRPC sequencer server failed")
            })
        };
        Self {
            _server: server,
            mock_server,
            local_addr,
        }
    }

    pub(crate) async fn mount_pending_nonce_response(
        &self,
        nonce_to_mount: u32,
        debug_name: impl Into<String>,
        expected_requests: u64,
    ) {
        let resp = GetPendingNonceResponse {
            inner: nonce_to_mount,
        };
        Mock::for_rpc_given(
            GET_PENDING_NONCE_GRPC_NAME,
            message_type::<GetPendingNonceRequest>(),
        )
        .respond_with(constant_response(resp))
        .up_to_n_times(1)
        .expect(expected_requests)
        .with_name(debug_name)
        .mount(&self.mock_server)
        .await;
    }
}

struct SequencerServiceImpl(MockServer);

#[tonic::async_trait]
impl SequencerService for SequencerServiceImpl {
    async fn get_sequencer_block(
        self: Arc<Self>,
        _request: Request<GetSequencerBlockRequest>,
    ) -> Result<Response<RawSequencerBlock>, Status> {
        unimplemented!()
    }

    async fn get_filtered_sequencer_block(
        self: Arc<Self>,
        _request: Request<GetFilteredSequencerBlockRequest>,
    ) -> Result<Response<RawFilteredSequencerBlock>, Status> {
        unimplemented!()
    }

    async fn get_pending_nonce(
        self: Arc<Self>,
        request: Request<GetPendingNonceRequest>,
    ) -> Result<Response<GetPendingNonceResponse>, Status> {
        self.0
            .handle_request(GET_PENDING_NONCE_GRPC_NAME, request)
            .await
    }

    async fn get_upgrades_info(
        self: Arc<Self>,
        _request: Request<GetUpgradesInfoRequest>,
    ) -> Result<Response<GetUpgradesInfoResponse>, Status> {
        unimplemented!()
    }

    async fn get_validator_name(
        self: Arc<Self>,
        _request: Request<GetValidatorNameRequest>,
    ) -> Result<Response<GetValidatorNameResponse>, Status> {
        unimplemented!()
    }
}
