use std::{
    net::SocketAddr,
    sync::Arc,
};

use astria_core::{
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
    primitive::v1::RollupId,
    protocol::test_utils::ConfigureSequencerBlock,
    sequencerblock::v1::{
        block,
        SequencerBlock,
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
    MockGuard,
    MockServer,
};
use tendermint::account::Id as AccountId;
use tokio::task::JoinHandle;
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

const GET_SEQUENCER_BLOCK_GRPC_NAME: &str = "get_sequencer_block";
const GET_FILTERED_SEQUENCER_BLOCK_GRPC_NAME: &str = "get_filtered_sequencer_block";

pub struct MockSequencerServer {
    _server: JoinHandle<eyre::Result<()>>,
    pub mock_server: MockServer,
    pub local_addr: SocketAddr,
}

impl MockSequencerServer {
    pub async fn spawn() -> Self {
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

    pub async fn mount_sequencer_block_response(
        &self,
        block_to_mount: SequencerBlockToMount,
        debug_name: impl Into<String>,
    ) {
        prepare_sequencer_block_response(block_to_mount, debug_name)
            .mount(&self.mock_server)
            .await;
    }

    pub async fn mount_sequencer_block_response_as_scoped(
        &self,
        block_to_mount: SequencerBlockToMount,
        debug_name: impl Into<String>,
    ) -> MockGuard {
        prepare_sequencer_block_response(block_to_mount, debug_name)
            .mount_as_scoped(&self.mock_server)
            .await
    }
}

pub enum SequencerBlockToMount {
    GoodAtHeight(u32),
    BadAtHeight(u32),
    Block(SequencerBlock),
}

struct SequencerServiceImpl(MockServer);

#[tonic::async_trait]
impl SequencerService for SequencerServiceImpl {
    async fn get_sequencer_block(
        self: Arc<Self>,
        request: Request<GetSequencerBlockRequest>,
    ) -> Result<Response<RawSequencerBlock>, Status> {
        self.0
            .handle_request(GET_SEQUENCER_BLOCK_GRPC_NAME, request)
            .await
    }

    async fn get_filtered_sequencer_block(
        self: Arc<Self>,
        request: Request<GetFilteredSequencerBlockRequest>,
    ) -> Result<Response<RawFilteredSequencerBlock>, Status> {
        self.0
            .handle_request(GET_FILTERED_SEQUENCER_BLOCK_GRPC_NAME, request)
            .await
    }

    async fn get_pending_nonce(
        self: Arc<Self>,
        _request: Request<GetPendingNonceRequest>,
    ) -> Result<Response<GetPendingNonceResponse>, Status> {
        unimplemented!()
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

fn prepare_sequencer_block_response(
    block_to_mount: SequencerBlockToMount,
    debug_name: impl Into<String>,
) -> Mock {
    let proposer = AccountId::try_from(vec![0u8; 20]).unwrap();
    let should_corrupt = matches!(block_to_mount, SequencerBlockToMount::BadAtHeight(_));

    let block = match block_to_mount {
        SequencerBlockToMount::GoodAtHeight(height)
        | SequencerBlockToMount::BadAtHeight(height) => ConfigureSequencerBlock {
            block_hash: Some(block::Hash::new([99u8; 32])),
            height,
            proposer_address: Some(proposer),
            sequence_data: vec![(
                RollupId::from_unhashed_bytes(b"some_rollup_id"),
                vec![99u8; 32],
            )],
            ..Default::default()
        }
        .make(),
        SequencerBlockToMount::Block(block) => block,
    };

    let mut block = block.into_raw();
    if should_corrupt {
        let header = block.header.as_mut().unwrap();
        let mut data_hash = header.data_hash.to_vec();
        data_hash[0] = data_hash[0].wrapping_add(1);
        header.data_hash = data_hash.into();
    }

    Mock::for_rpc_given(
        GET_SEQUENCER_BLOCK_GRPC_NAME,
        message_type::<GetSequencerBlockRequest>(),
    )
    .respond_with(constant_response(block))
    .up_to_n_times(1)
    .expect(1)
    .with_name(debug_name)
}
