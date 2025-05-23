use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    io::Write,
    net::{
        IpAddr,
        SocketAddr,
    },
    sync::LazyLock,
    time::Duration,
};

use astria_composer::{
    config::Config,
    Composer,
    Metrics,
};
use astria_core::{
    generated::astria::protocol::accounts::v1::NonceResponse,
    primitive::v1::{
        asset::{
            Denom,
            IbcPrefixed,
        },
        RollupId,
    },
    protocol::{
        abci::AbciErrorCode,
        transaction::v1::Transaction,
    },
    Protobuf as _,
};
use astria_eyre::eyre;
use ethers::{
    prelude::Transaction as EthersTransaction,
    types::H160,
};
use mock_grpc_sequencer::MockGrpcSequencer;
use prost::Message as _;
use serde_json::json;
use telemetry::metrics;
use tempfile::NamedTempFile;
use tendermint_rpc::{
    endpoint::broadcast::tx_sync,
    request,
    response,
    Id,
};
use test_utils::mock::Geth;
use tokio::task::JoinHandle;
use tracing::debug;
use wiremock::{
    matchers::{
        body_partial_json,
        body_string_contains,
    },
    Mock,
    MockGuard,
    MockServer,
    Request,
    ResponseTemplate,
};

pub mod mock_abci_sequencer;
pub mod mock_grpc_sequencer;

pub const TEST_CHAIN_ID: &str = "test-chain-1";

static TELEMETRY: LazyLock<()> = LazyLock::new(|| {
    // This config can be meaningless - it's only used inside `try_init` to init the metrics, but we
    // haven't configured telemetry to provide metrics here.
    let config = Config {
        log: String::new(),
        api_listen_addr: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0),
        sequencer_abci_endpoint: String::new(),
        sequencer_grpc_endpoint: String::new(),
        sequencer_chain_id: String::new(),
        rollups: String::new(),
        private_key_file: String::new(),
        sequencer_address_prefix: String::new(),
        block_time_ms: 0,
        max_bytes_per_bundle: 0,
        bundle_queue_capacity: 0,
        force_stdout: false,
        no_otel: false,
        no_metrics: false,
        metrics_http_listener_addr: String::new(),
        grpc_addr: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0),
        fee_asset: Denom::IbcPrefixed(IbcPrefixed::new([0; 32])),
    };
    if std::env::var_os("TEST_LOG").is_some() {
        let filter_directives = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
        telemetry::configure()
            .set_no_otel(true)
            .set_force_stdout(true)
            .set_filter_directives(&filter_directives)
            .try_init::<Metrics>(&config)
            .unwrap();
    } else {
        telemetry::configure()
            .set_no_otel(true)
            .set_stdout_writer(std::io::sink)
            .try_init::<Metrics>(&config)
            .unwrap();
    }
});

pub struct TestComposer {
    pub cfg: Config,
    pub composer: JoinHandle<eyre::Result<()>>,
    pub rollup_nodes: HashMap<String, Geth>,
    pub sequencer: wiremock::MockServer,
    pub sequencer_mock: MockGrpcSequencer,
    pub grpc_collector_addr: SocketAddr,
    pub metrics_handle: metrics::Handle,
}

/// Spawns composer in a test environment.
///
/// # Panics
/// There is no explicit error handling in favour of panicking loudly
/// and early.
pub async fn spawn_composer(
    rollup_ids: &[&str],
    sequencer_chain_id: Option<&str>,
    txs_in_pool: Vec<EthersTransaction>,
    loop_until_ready: bool,
) -> TestComposer {
    LazyLock::force(&TELEMETRY);

    let mut rollup_nodes = HashMap::new();
    let mut rollups = String::new();
    for id in rollup_ids {
        let mut pending_map = BTreeMap::new();
        let mut nonce_map = BTreeMap::new();
        for (index, tx) in txs_in_pool.iter().enumerate() {
            nonce_map.insert(index.to_string(), tx.clone());
        }
        pending_map.insert(H160::zero(), nonce_map);
        let geth = Geth::spawn_with_pending_txs(pending_map).await;
        let execution_url = format!("ws://{}", geth.local_addr());
        rollup_nodes.insert((*id).to_string(), geth);
        rollups.push_str(&format!("{id}::{execution_url},"));
    }
    let sequencer = mock_abci_sequencer::start(sequencer_chain_id).await;
    let grpc_server = MockGrpcSequencer::spawn().await;
    let sequencer_url = sequencer.uri();
    let keyfile = NamedTempFile::new().unwrap();
    (&keyfile)
        .write_all("2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90".as_bytes())
        .unwrap();
    let config = Config {
        log: String::new(),
        api_listen_addr: "127.0.0.1:0".parse().unwrap(),
        sequencer_chain_id: TEST_CHAIN_ID.to_string(),
        rollups,
        sequencer_abci_endpoint: sequencer_url.to_string(),
        sequencer_grpc_endpoint: format!("http://{}", grpc_server.local_addr),
        private_key_file: keyfile.path().to_string_lossy().to_string(),
        sequencer_address_prefix: "astria".into(),
        block_time_ms: 2000,
        max_bytes_per_bundle: 200_000,
        bundle_queue_capacity: 10,
        no_otel: false,
        force_stdout: false,
        no_metrics: true,
        metrics_http_listener_addr: String::new(),
        grpc_addr: "127.0.0.1:0".parse().unwrap(),
        fee_asset: "nria".parse().unwrap(),
    };

    let (metrics, metrics_handle) = metrics::ConfigBuilder::new()
        .set_global_recorder(false)
        .build(&config)
        .unwrap();
    let metrics = Box::leak(Box::new(metrics));

    let expected_get_nonce_requests = loop_until_ready.into();

    // prepare get nonce response
    grpc_server
        .mount_pending_nonce_response(
            0,
            "startup::wait_for_mempool()",
            expected_get_nonce_requests,
        )
        .await;

    let (composer_addr, grpc_collector_addr, composer_handle) = {
        let composer = Composer::from_config(&config, metrics).await.unwrap();
        let composer_addr = composer.local_addr();
        let grpc_collector_addr = composer.grpc_local_addr().unwrap();
        let task = tokio::spawn(composer.run_until_stopped());
        (composer_addr, grpc_collector_addr, task)
    };

    if loop_until_ready {
        loop_until_composer_is_ready(composer_addr).await;
    }

    TestComposer {
        cfg: config,
        composer: composer_handle,
        rollup_nodes,
        sequencer,
        sequencer_mock: grpc_server,
        grpc_collector_addr,
        metrics_handle,
    }
}

/// Query composer's `/readyz` endpoint until its ready.
///
/// # Panics
///
/// Panics instead of handling errors if no HTTP request could be sent to
/// composer or if its response could not be deserialized as JSON.
pub async fn loop_until_composer_is_ready(addr: SocketAddr) {
    #[derive(Debug, serde::Deserialize)]
    struct Readyz {
        status: String,
    }

    loop {
        let readyz = reqwest::get(format!("http://{addr}/readyz"))
            .await
            .unwrap()
            .json::<Readyz>()
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        if readyz.status.to_lowercase() == "ok" {
            break;
        }
    }
}

/// Creates a signed transaction from a wiremock request.
///
/// # Panics
///
/// Panics if the request body can't be deserialiezed to a JSONRPC wrapped `tx_sync::Request`, or if
/// the deserialization from the JSONRPC request to the raw transaction fails.
pub fn signed_tx_from_request(request: &Request) -> Transaction {
    use astria_core::generated::astria::protocol::transaction::v1::Transaction as RawTransaction;
    use prost::Message as _;

    let wrapped_tx_sync_req: request::Wrapper<tx_sync::Request> =
        serde_json::from_slice(&request.body)
            .expect("can't deserialize to JSONRPC wrapped tx_sync::Request");
    let raw_tx = RawTransaction::decode(&*wrapped_tx_sync_req.params().tx)
        .expect("can't deserialize sequencer tx from broadcast jsonrpc request");
    let tx = Transaction::try_from_raw(raw_tx).expect("can't validate raw sequencer tx");
    debug!(?tx, "sequencer mock received transaction");
    tx
}

fn rollup_id_nonce_from_request(request: &Request) -> (RollupId, u32) {
    let signed_tx = signed_tx_from_request(request);

    // validate that the transaction's first action is a sequence action
    let Some(sent_action) = signed_tx.actions().first() else {
        panic!("received transaction contained no actions");
    };
    let Some(sequence_action) = sent_action.as_rollup_data_submission() else {
        panic!("mocked sequencer expected a sequence action");
    };

    (sequence_action.rollup_id, signed_tx.nonce())
}

/// Deserializes the bytes contained in a `tx_sync::Request` to a signed sequencer transaction and
/// verifies that it contains a sequence action with `expected_payload` as its contents.
/// # Panics
/// Panics if the request body has no sequence actions
pub async fn mount_matcher_verifying_tx_integrity(
    server: &MockServer,
    expected_rlp: EthersTransaction,
) -> MockGuard {
    let matcher = move |request: &Request| {
        let sequencer_tx = signed_tx_from_request(request);
        let rollup_data_submission = sequencer_tx
            .actions()
            .first()
            .unwrap()
            .as_rollup_data_submission()
            .unwrap();

        let expected_rlp = expected_rlp.rlp().to_vec();

        expected_rlp == rollup_data_submission.data
    };
    let jsonrpc_rsp = response::Wrapper::new_with_id(
        Id::Num(1),
        Some(tx_sync::Response {
            code: 0.into(),
            data: vec![].into(),
            log: String::new(),
            hash: tendermint::Hash::Sha256([0; 32]),
            codespace: String::new(),
        }),
        None,
    );

    Mock::given(matcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(&jsonrpc_rsp))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(server)
        .await
}

/// Deserializes the bytes contained in a `tx_sync::Request` to a signed sequencer transaction and
/// verifies that the contained sequence action is in the given `expected_rollup_ids` and
/// `expected_nonces`.
/// # Panics
/// Panics if the expected nonces vector is empty
pub async fn mount_broadcast_tx_sync_mock(
    server: &MockServer,
    expected_rollup_ids: Vec<RollupId>,
    expected_nonces: Vec<u32>,
) -> MockGuard {
    let expected_calls = expected_nonces.len().try_into().unwrap();
    let matcher = move |request: &Request| {
        let (rollup_id, nonce) = rollup_id_nonce_from_request(request);

        let valid_rollup_id = expected_rollup_ids.contains(&rollup_id);
        let valid_nonce = expected_nonces.contains(&nonce);

        valid_rollup_id && valid_nonce
    };
    let jsonrpc_rsp = response::Wrapper::new_with_id(
        Id::Num(1),
        Some(tx_sync::Response {
            code: 0.into(),
            data: vec![].into(),
            log: String::new(),
            hash: tendermint::Hash::Sha256([0; 32]),
            codespace: String::new(),
        }),
        None,
    );

    Mock::given(matcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(&jsonrpc_rsp))
        .up_to_n_times(expected_calls)
        .expect(expected_calls)
        .mount_as_scoped(server)
        .await
}

/// Deserializes the bytes contained in a `tx_sync::Request` to a signed sequencer transaction and
/// verifies that the contained sequence action is for the given `expected_rollup_id`. It then
/// rejects the transaction for an invalid nonce.
pub async fn mount_broadcast_tx_sync_invalid_nonce_mock(
    server: &MockServer,
    expected_rollup_id: RollupId,
) -> MockGuard {
    let matcher = move |request: &Request| {
        let (rollup_id, _) = rollup_id_nonce_from_request(request);
        rollup_id == expected_rollup_id
    };
    let jsonrpc_rsp = response::Wrapper::new_with_id(
        Id::Num(1),
        Some(tx_sync::Response {
            code: tendermint::abci::Code::Err(AbciErrorCode::INVALID_NONCE.value()),
            data: vec![].into(),
            log: String::new(),
            hash: tendermint::Hash::Sha256([0; 32]),
            codespace: String::new(),
        }),
        None,
    );
    Mock::given(matcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(&jsonrpc_rsp))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(server)
        .await
}

/// Deserializes the bytes contained in a `tx_sync::Request` to a signed sequencer transaction and
/// verifies that the contained sequence action is for the given `expected_rollup_id`. It then
/// rejects the transaction for a taken nonce.
pub async fn mount_broadcast_tx_sync_nonce_taken_mock(
    server: &MockServer,
    expected_rollup_id: RollupId,
) -> MockGuard {
    let matcher = move |request: &Request| {
        let (rollup_id, _) = rollup_id_nonce_from_request(request);
        rollup_id == expected_rollup_id
    };
    let jsonrpc_rsp = response::Wrapper::new_with_id(
        Id::Num(1),
        Some(tx_sync::Response {
            code: tendermint::abci::Code::Err(AbciErrorCode::NONCE_TAKEN.value()),
            data: vec![].into(),
            log: String::new(),
            hash: tendermint::Hash::Sha256([0; 32]),
            codespace: String::new(),
        }),
        None,
    );
    Mock::given(matcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(&jsonrpc_rsp))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(server)
        .await
}

/// Deserializes the bytes contained in a `tx_sync::Request` to a signed sequencer transaction
/// and verifies that the contained rollup data submission action is in the given
/// `expected_rollup_ids` and `expected_nonces`.
pub async fn mount_broadcast_tx_sync_rollup_data_submissions_mock(
    server: &MockServer,
) -> MockGuard {
    let matcher = move |request: &Request| {
        let signed_tx = signed_tx_from_request(request);
        let actions = signed_tx.actions();

        // verify all received actions are sequence actions
        actions
            .iter()
            .all(|action| action.as_rollup_data_submission().is_some())
    };
    let jsonrpc_rsp = response::Wrapper::new_with_id(
        Id::Num(1),
        Some(tx_sync::Response {
            code: 0.into(),
            data: vec![].into(),
            log: String::new(),
            hash: tendermint::Hash::Sha256([0; 32]),
            codespace: String::new(),
        }),
        None,
    );

    Mock::given(matcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(&jsonrpc_rsp))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(server)
        .await
}

/// Mount a mock for the `abci_query` endpoint.
pub async fn mount_default_nonce_query_mock(server: &MockServer) -> MockGuard {
    let query_path = "accounts/nonce";
    let response = NonceResponse {
        height: 0,
        nonce: 0,
    };
    let expected_body = json!({
        "method": "abci_query"
    });
    let response = tendermint_rpc::endpoint::abci_query::Response {
        response: tendermint_rpc::endpoint::abci_query::AbciQuery {
            value: response.encode_to_vec(),
            ..Default::default()
        },
    };
    let wrapper = response::Wrapper::new_with_id(Id::Num(1), Some(response), None);
    Mock::given(body_partial_json(&expected_body))
        .and(body_string_contains(query_path))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&wrapper)
                .append_header("Content-Type", "application/json"),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(server)
        .await
}

// A Uniswap V2 DAI-ETH swap transaction from mainnet
// Etherscan link: https://etherscan.io/tx/0x99850dd1cf325c8ede9ba62b9d8a11aa199794450b581ce3a7bb8c1e5bb7562f
pub const TEST_ETH_TX_JSON: &str = r#"{"blockHash":"0xe365f2163edb844b617ebe3d2af183b31d6c7ffa794f21d0b2d111d63e979a02","blockNumber":"0x1157959","from":"0xdc975a9bb00f4c030e4eb3268f68e4b8d0fa0362","gas":"0xcdf49","gasPrice":"0x374128344","maxFeePerGas":"0x374128344","maxPriorityFeePerGas":"0x0","hash":"0x99850dd1cf325c8ede9ba62b9d8a11aa199794450b581ce3a7bb8c1e5bb7562f","input":"0x022c0d9f0000000000000000000000000000000000000000000000c88a1ad5e15105525500000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a2d11cb90d1de13bb81ee7b772a08ac234a8058000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001208974000000000000000000000000000000000000000000000000000000004de4000000000000000000000000000000000000000000000000017038152c223cb100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000005200000000000000000000000000000000000000000000000000000000000000000000000000000000000000087870bca3f3fd6335c3f4ce8392d69350b4fa4e2000000000000000000000000ab12275f2d91f87b301a4f01c9af4e83b3f45baa0000000000000000000000006b175474e89094c44da98b954eedeac495271d0f000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","nonce":"0x28","to":"0xa478c2975ab1ea89e8196811f51a7b7ade33eb11","transactionIndex":"0x2","value":"0x0","type":"0x2","accessList":[{"address":"0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000005","0x0000000000000000000000000000000000000000000000000000000000000002"]},{"address":"0x7effd7b47bfd17e52fb7559d3f924201b9dbff3d","storageKeys":[]},{"address":"0x018008bfb33d285247a21d44e50697654f754e63","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc"]},{"address":"0x1a2d11cb90d1de13bb81ee7b772a08ac234a8058","storageKeys":[]},{"address":"0xe62b71cf983019bff55bc83b48601ce8419650cc","storageKeys":["0x9a09f352b299559621084d9b8d2625e8d5a97f382735872dd3bb1bdbdccc3fee","0x000000000000000000000000000000000000000000000000000000000000002b","0xfee3a99380070b792e111dd9a6a15e929983e2d0b7e170a5520e51b99be0c359"]},{"address":"0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x070a95ec3546cae47592e0bcea195bf8f96287077fbb7a23785cc2887152941c","0x070a95ec3546cae47592e0bcea195bf8f96287077fbb7a23785cc28871529420","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec6","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4b","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ebf","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec0","0x4c0bd942d17410ca1f6d3278a62feef7078602605466e37de958808f1454efbd","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e48","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec3","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4f","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4a","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e50","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4d","0x4cb2b152c1b54ce671907a93c300fd5aa72383a9d4ec19a81e3333632ae92e00","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec4","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec7","0x4bea7244bd9088ac961c659a818b4f060de9712d20dc006c24f0985f19cf62d1","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e49","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec2","0x070a95ec3546cae47592e0bcea195bf8f96287077fbb7a23785cc2887152941d","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4c","0x5e14560e314427eb9d0c466a6058089f672317c8e26719a770a709c3f2481e4e","0x4480713a5820391a4815a640728dab70c3847e45854ef9e8117382da26ce9105","0x070a95ec3546cae47592e0bcea195bf8f96287077fbb7a23785cc2887152941f","0x000000000000000000000000000000000000000000000000000000000000003b","0x108718ddd11d4cf696a068770009c44aef387eb858097a37824291f99278d5e3","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec1","0xf81d8d79f42adb4c73cc3aa0c78e25d3343882d0313c0b80ece3d3a103ef1ec5"]},{"address":"0x2f39d218133afab8f2b819b1066c7e434ad94e9e","storageKeys":["0x740f710666bd7a12af42df98311e541e47f7fd33d382d11602457a6d540cbd63","0x0d2c1bcee56447b4f46248272f34207a580a5c40f666a31f4e2fbb470ea53ab8"]},{"address":"0xe7b67f44ea304dd7f6d215b13686637ff64cd2b2","storageKeys":[]},{"address":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","storageKeys":["0x7f6377583d24615ddfe989626525aeed0d158f924ee8c91664ab0dffd7863d00","0x3afb575d989d656a39ee0690da12b019915f3bd8709cc522e681b8dd04237970","0xa535fbd0ab3e0ad4ee444570368f3d474545b71fcc49228fe96a6406676fc126","0xb064600732a82908427d092d333e607598a6238a59aeb45e1288cb0bac7161cf"]},{"address":"0x4d5f47fa6a74757f35c14fd3a6ef8e3c9bc514e8","storageKeys":["0x000000000000000000000000000000000000000000000000000000000000003c","0x14a553e31736f19e3e380cf55bfb2f82dfd6d880cd07235affb68d8d3e0cac4d","0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x5e8cc6ee686108b7fd15638e2dbb32555b30d0bd1a191628bb70b5459b86cedc","0x000000000000000000000000000000000000000000000000000000000000003d","0x0000000000000000000000000000000000000000000000000000000000000036","0x0000000000000000000000000000000000000000000000000000000000000039"]},{"address":"0x6b175474e89094c44da98b954eedeac495271d0f","storageKeys":["0xd86cc1e239204d48eb0055f151744c4bb3d2337612287be803ae8247e95a67d2","0xe7ab5c3b3c86286a122f1937d4c70a3170dba7ef4f7603d830e8bcf7c9af583b","0x87c358b8e65d7446f52ffce25e44c9673d2bf461b3d3e4748afcf1238e9224a3","0xad740bfd58072c0bd719418966c52da18e837afec1b47e07bba370568cc87fbb"]},{"address":"0xe175de51f29d822b86e46a9a61246ec90631210d","storageKeys":[]},{"address":"0xcf8d0c70c850859266f5c338b38f9d663181c314","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000037","0x000000000000000000000000000000000000000000000000000000000000003d","0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x000000000000000000000000000000000000000000000000000000000000003a","0x4bea7244bd9088ac961c659a818b4f060de9712d20dc006c24f0985f19cf62d1"]},{"address":"0x413adac9e2ef8683adf5ddaece8f19613d60d1bb","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x000000000000000000000000000000000000000000000000000000000000003f","0x000000000000000000000000000000000000000000000000000000000000003a","0x4bea7244bd9088ac961c659a818b4f060de9712d20dc006c24f0985f19cf62d1"]},{"address":"0xaed0c38402a5d19df6e4c03f4e2dced6e29c1ee9","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000005","0x0000000000000000000000000000000000000000000000000000000000000002"]},{"address":"0xea51d7853eefb32b6ee06b1c12e6dcca88be0ffe","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x000000000000000000000000000000000000000000000000000000000000003a"]},{"address":"0x54586be62e3c3580375ae3723c145253060ca0c2","storageKeys":["0x7145bb02480b505fc02ccfdba07d3ba3a9d821606f0688263abedd0ac6e5bec5","0x2a11cb67ca5c7e99dba99b50e02c11472d0f19c22ed5af42a1599a7f57e1c7a4","0x5306b8fbe80b30a74098357ee8e26fad8dc069da9011cca5f0870a0a5982e541"]},{"address":"0x478238a1c8b862498c74d0647329aef9ea6819ed","storageKeys":["0x9ef04667c5a1bd8192837ceac2ad5f2c41549d4db3406185e8c6aa95ea557bc5","0x000000000000000000000000000000000000000000000000000000000000002b","0x0020b304a2489d03d215fadd3bb6d3de2dda5a6a1235e76d693c30263e3cd054"]},{"address":"0xa700b4eb416be35b2911fd5dee80678ff64ff6c9","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x5e8cc6ee686108b7fd15638e2dbb32555b30d0bd1a191628bb70b5459b86cedc"]},{"address":"0x8164cc65827dcfe994ab23944cbc90e0aa80bfcb","storageKeys":["0x76f8b43dabb591eb6681562420f7f6aa393e6903d4e02e6f59e2957d94ceab20","0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x176062dac4e737f036c34baf4b07185f9c9fd3c1337ca36eb7c1f7a74aedb8ea"]},{"address":"0x9a158802cd924747ef336ca3f9de3bdb60cf43d3","storageKeys":[]},{"address":"0xac725cb59d16c81061bdea61041a8a5e73da9ec6","storageKeys":[]},{"address":"0x15c5620dffac7c7366eed66c20ad222ddbb1ed57","storageKeys":[]},{"address":"0x547a514d5e3769680ce22b2361c10ea13619e8a9","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000005","0x0000000000000000000000000000000000000000000000000000000000000002"]},{"address":"0x8116b273cd75d79c382afacc706659ded5e0a59d","storageKeys":["0x0fb35ae12d348b84dc0910bcce7d3b0a3f6d23a3e1d0b53bbe5f135078b97b13","0x000000000000000000000000000000000000000000000000000000000000002b","0x1d90d8e683e6736ac0564a19732a642e4be100e7ee8c225feba909bbdaf1522b"]},{"address":"0x9f8ccdafcc39f3c7d6ebf637c9151673cbc36b88","storageKeys":[]},{"address":"0xa478c2975ab1ea89e8196811f51a7b7ade33eb11","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000007","0x0000000000000000000000000000000000000000000000000000000000000009","0x000000000000000000000000000000000000000000000000000000000000000a","0x000000000000000000000000000000000000000000000000000000000000000c","0x0000000000000000000000000000000000000000000000000000000000000008","0x0000000000000000000000000000000000000000000000000000000000000006"]},{"address":"0xf1cd4193bbc1ad4a23e833170f49d60f3d35a621","storageKeys":[]},{"address":"0x102633152313c81cd80419b6ecf66d14ad68949a","storageKeys":["0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc","0x000000000000000000000000000000000000000000000000000000000000003f","0x000000000000000000000000000000000000000000000000000000000000003a"]},{"address":"0xb02381b1d27aa9845e5012083ca288c1818884f0","storageKeys":[]}],"chainId":"0x1","v":"0x0","r":"0xcb4eccf09e298388220c5560a6539322bde17581cee6908d56a92a19575e28e2","s":"0x2b4e34adad48aee14b6600c6366ad683c00c63c9da88fc2a232308421cf69a21"}"#;
