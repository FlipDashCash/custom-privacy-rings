//! Real loopback HTTP, SQLite persistence and child-process SIGKILL boundaries.
//! The chain/reviewer are deliberately fake: this is NOT proof-bearing E2E.
#[allow(dead_code)]
#[path = "common/relay.rs"]
mod relay_fixture;

use async_trait::async_trait;
use relay_fixture::{RecordingChain, RecordingProver};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::VersionedMessage;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spr_client::{compile_message, ComputeBudgetConfig};
use spr_indexer_api::Base64String;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zolana_ring_client::rpc::ServiceTarget;
use zolana_ring_rpc::{
    relay_module, ChainOutcome, ChainSubmit, LatestBlockhash, OrdinaryProofOnlyContext,
    OrdinaryProofOnlyPolicy, OrdinaryProofOnlyReview, PrepareOrdinaryProofOnlyRequest, Relay,
    RelayBudget, RelayParts, RingRpcError, ServedRings, Simulation, Store,
};

const RING: Address = Address::new_from_array([6; 32]);
const SERVICE: ServiceTarget = ServiceTarget {
    cluster: [7; 32],
    service: Address::new_from_array([9; 32]),
};
const WORKER_DIRECTORY: &str = "PYROL_PROOF_ONLY_HTTP_WORKER_DIRECTORY";
const WORKER_WINDOW: &str = "PYROL_PROOF_ONLY_HTTP_WORKER_WINDOW";

struct Policy(OrdinaryProofOnlyContext);
#[async_trait]
impl OrdinaryProofOnlyPolicy for Policy {
    fn context(&self) -> &OrdinaryProofOnlyContext {
        &self.0
    }
    async fn review(&self, wire: &Base64String) -> Result<OrdinaryProofOnlyReview, RingRpcError> {
        let tx: VersionedTransaction = wincode::deserialize(&wire.0).expect("fixture transaction");
        Ok(OrdinaryProofOnlyReview {
            ring: RING.into(),
            policy_sha256: self.0.policy_sha256.clone(),
            message_sha256: hex::encode(Sha256::digest(tx.message.serialize())),
            last_valid_block_height: "500".into(),
            maximum_debit_lamports: "100000".into(),
            nullifiers: vec!["11".repeat(32), "22".repeat(32)],
        })
    }
}
struct Rings;
#[async_trait]
impl ServedRings for Rings {
    async fn check(&self, ring: Address) -> Result<(), RingRpcError> {
        assert_eq!(ring, RING);
        Ok(())
    }
    async fn delegate(&self, _: Address) -> Result<Option<Address>, RingRpcError> {
        Ok(None)
    }
}

fn durable(path: &Path, bytes: &[u8]) {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .expect("new evidence file");
    file.write_all(bytes).expect("write evidence");
    file.sync_all().expect("sync evidence");
}

/// Its disk receipt stands in for a chain that survives the backend process.
struct FileChain {
    inner: RecordingChain,
    directory: PathBuf,
    window: String,
}
#[async_trait]
impl ChainSubmit for FileChain {
    async fn slot(&self) -> Result<u64, RingRpcError> {
        self.inner.slot().await
    }
    async fn latest_blockhash(&self) -> Result<LatestBlockhash, RingRpcError> {
        self.inner.latest_blockhash().await
    }
    async fn block_height(&self) -> Result<u64, RingRpcError> {
        self.inner.block_height().await
    }
    async fn account_data(&self, address: Address) -> Result<Option<Vec<u8>>, RingRpcError> {
        self.inner.account_data(address).await
    }
    async fn rent_exempt_lamports(&self, len: usize) -> Result<u64, RingRpcError> {
        self.inner.rent_exempt_lamports(len).await
    }
    async fn simulate(&self, message: &VersionedMessage) -> Result<Simulation, RingRpcError> {
        self.inner.simulate(message).await
    }
    async fn send(&self, transaction: &VersionedTransaction) -> Result<(), RingRpcError> {
        // Both callers (the request and background tracker) block at the same boundary.
        if self.window == "sent-before-network" {
            let marker = self.directory.join("crash-boundary");
            if !marker.exists() {
                durable(&marker, b"sent-before-network");
            }
            std::future::pending::<()>().await;
        }
        let wire = wincode::serialize(transaction).expect("public wire");
        let canonical = self.directory.join("fake-chain-wire.bin");
        if canonical.exists() {
            assert_eq!(
                std::fs::read(&canonical).unwrap(),
                wire,
                "never replace the original"
            );
        } else {
            durable(&canonical, &wire);
        }
        let mut calls = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.directory.join("send-calls"))
            .unwrap();
        calls.write_all(b"send\n").unwrap();
        calls.sync_all().unwrap();
        if self.window == "after-network-before-response" {
            let marker = self.directory.join("crash-boundary");
            if !marker.exists() {
                durable(&marker, b"after-network-before-response");
            }
            std::future::pending::<()>().await;
        }
        Ok(())
    }
    async fn outcome(&self, signature: Signature) -> Result<Option<ChainOutcome>, RingRpcError> {
        let path = self.directory.join("fake-chain-wire.bin");
        if !path.exists() {
            return Ok(None);
        }
        let tx: VersionedTransaction = wincode::deserialize(&std::fs::read(path).unwrap()).unwrap();
        Ok((tx.signatures[0] == signature).then_some(ChainOutcome::Confirmed { slot: 1000 }))
    }
}

fn context() -> OrdinaryProofOnlyContext {
    OrdinaryProofOnlyContext {
        genesis_hash: Address::new_from_array(SERVICE.cluster).to_string(),
        ring: RING.into(),
        tree: Address::new_from_array([8; 32]).into(),
        payer: Keypair::new_from_array([31; 32]).pubkey().into(),
        policy_sha256: "ab".repeat(32),
    }
}
fn authorize(action: &str, wire: &Base64String, signed_hash: &str) -> String {
    let c = context();
    let key = Keypair::new_from_array([32; 32]); // Public, deterministic test-only fixture.
    let tx: VersionedTransaction = wincode::deserialize(&wire.0).unwrap();
    let bytes = format!(
        "pyrol/ordinary-proof-only/v1\n{action}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{signed_hash}",
        c.genesis_hash,
        c.ring.0,
        c.tree.0,
        c.payer.0,
        c.policy_sha256,
        hex::encode(Sha256::digest(tx.message.serialize())),
        key.pubkey()
    );
    key.sign_message(bytes.as_bytes()).to_string()
}
async fn request() -> PrepareOrdinaryProofOnlyRequest {
    let c = context();
    let chain = RecordingChain::default();
    let tx = VersionedTransaction {
        message: compile_message(
            &c.payer.0,
            &[Instruction {
                program_id: RING,
                accounts: vec![],
                data: vec![55],
            }],
            chain.latest_blockhash().await.unwrap().blockhash,
            ComputeBudgetConfig::new(400000),
        )
        .unwrap(),
        signatures: vec![Signature::default()],
    };
    let wire = Base64String(wincode::serialize(&tx).unwrap());
    PrepareOrdinaryProofOnlyRequest {
        authorization_signature: authorize("prepare", &wire, ""),
        transaction: wire,
        authorization_public_key: Keypair::new_from_array([32; 32]).pubkey().into(),
        policy_sha256: c.policy_sha256,
    }
}

#[test]
#[ignore = "internal subprocess worker; the three HTTP tests invoke it"]
fn proof_only_http_crash_worker() {
    let directory =
        PathBuf::from(std::env::var(WORKER_DIRECTORY).expect("parent-owned test directory"));
    let window = std::env::var(WORKER_WINDOW).expect("crash window");
    assert!(matches!(
        window.as_str(),
        "prepared" | "sent-before-network" | "after-network-before-response" | "restart"
    ));
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let database = format!(
            "sqlite://{}?mode=rwc",
            directory.join("relay.sqlite").display()
        );
        let store = Store::open(&database, 1).await.unwrap();
        let chain = Arc::new(FileChain {
            inner: RecordingChain::default(),
            directory: directory.clone(),
            window: window.clone(),
        });
        let relay = Arc::new(
            Relay::new(
                RelayParts {
                    store,
                    chain,
                    prover: Arc::new(RecordingProver::default()),
                    fee_payer: Keypair::new_from_array([31; 32]),
                },
                SERVICE,
            )
            .with_budget(RelayBudget {
                total_lamports: 1000000,
                signer_lamports: 1000000,
            })
            .with_rings(Arc::new(Rings))
            .with_ordinary_proof_only_policy(Arc::new(Policy(context()))),
        );
        let server = jsonrpsee::server::ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .unwrap();
        let address = server.local_addr().unwrap();
        let handle = server.start(relay_module(relay.clone()).unwrap());
        // Initial process has no tracker: this makes each injection boundary exact.
        // The independently reopened backend exercises the production tracker.
        if window == "restart" {
            tokio::spawn(relay.run(Duration::from_millis(25)));
        }
        durable(
            &directory.join(format!("{window}.ready")),
            format!("http://{address}").as_bytes(),
        );
        handle.stopped().await;
    });
}

struct Worker(Child);
impl Worker {
    fn start(directory: &Path, window: &str) -> Self {
        let log = std::fs::File::create(directory.join(format!("{window}.log"))).unwrap();
        Self(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "proof_only_http_crash_worker",
                    "--ignored",
                    "--nocapture",
                ])
                .env(WORKER_DIRECTORY, directory)
                .env(WORKER_WINDOW, window)
                .stdin(Stdio::null())
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap(),
        )
    }
    fn kill_and_wait(&mut self) {
        use std::os::unix::process::ExitStatusExt;
        self.0.kill().expect("kill owned child");
        assert_eq!(
            self.0.wait().unwrap().signal(),
            Some(9),
            "SIGKILL, not orderly shutdown"
        );
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_file(path: &Path) -> Vec<u8> {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Ok(bytes) = std::fs::read(path) {
                if !bytes.is_empty() {
                    return bytes;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("worker boundary timeout")
}
async fn rpc(url: &str, method: &str, params: Value) -> Result<Value, reqwest::Error> {
    reqwest::Client::new()
        .post(url)
        .json(&json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params}))
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
fn result(value: Value) -> Value {
    assert!(value.get("error").is_none(), "RPC refused: {value}");
    value.get("result").expect("JSON-RPC result").clone()
}

async fn crash_case(window: &str) {
    let parent = std::env::var_os("PYROL_BACKEND_TEST_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = parent.join(format!(
        "pyrol-http-{window}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let mut worker = Worker::start(&directory, window);
    let url =
        String::from_utf8(wait_file(&directory.join(format!("{window}.ready"))).await).unwrap();
    let request = request().await;
    let prepared = result(
        rpc(
            &url,
            "prepareOrdinaryProofOnly",
            serde_json::to_value(&request).unwrap(),
        )
        .await
        .unwrap(),
    );
    let wire: Base64String = serde_json::from_value(prepared["transaction"].clone()).unwrap();
    let tx: VersionedTransaction = wincode::deserialize(&wire.0).unwrap();
    assert!(tx.signatures[0].verify(context().payer.0.as_ref(), &tx.message.serialize()));
    assert!(
        !directory.join("fake-chain-wire.bin").exists(),
        "prepare must not send"
    );
    durable(&directory.join("original-public-wire.bin"), &wire.0);
    let commit = json!({"id":prepared["id"], "wireSha256":hex::encode(Sha256::digest(&wire.0)),
        "authorizationPublicKey":request.authorization_public_key,
        "authorizationSignature":authorize("commit", &request.transaction, &hex::encode(Sha256::digest(&wire.0))), "policySha256":request.policy_sha256});
    if window == "prepared" {
        worker.kill_and_wait();
    } else {
        let pending =
            tokio::spawn(async move { rpc(&url, "commitOrdinaryProofOnly", commit).await });
        wait_file(&directory.join("crash-boundary")).await;
        worker.kill_and_wait();
        assert!(
            pending.await.unwrap().is_err(),
            "the killed request must not claim completion"
        );
    }
    if window == "sent-before-network" {
        assert!(!directory.join("fake-chain-wire.bin").exists());
    }
    let mut restarted = Worker::start(&directory, "restart");
    let url = String::from_utf8(wait_file(&directory.join("restart.ready")).await).unwrap();
    let recovered = result(rpc(&url, "recoverOrdinaryProofOnly", json!({"id":prepared["id"],
        "authorizationPublicKey":request.authorization_public_key,
        "authorizationSignature":authorize("recover", &request.transaction, ""), "policySha256":request.policy_sha256})).await.unwrap());
    assert_eq!(
        recovered["transaction"], prepared["transaction"],
        "restart recovers exact signed original"
    );
    if window == "prepared" {
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(
            !directory.join("fake-chain-wire.bin").exists(),
            "recovery and tracker never commit prepared wire"
        );
        let hash = hex::encode(Sha256::digest(&wire.0));
        result(rpc(&url, "commitOrdinaryProofOnly", json!({"id":prepared["id"], "wireSha256":hash,
            "authorizationPublicKey":request.authorization_public_key,
            "authorizationSignature":authorize("commit", &request.transaction, &hex::encode(Sha256::digest(&wire.0))),
            "policySha256":request.policy_sha256})).await.unwrap());
    }
    assert_eq!(
        wait_file(&directory.join("fake-chain-wire.bin")).await,
        wire.0
    );
    // The legacy unauthenticated status endpoint intentionally refuses this
    // protocol. Audit the owned test database, do not weaken that boundary.
    let refused = rpc(&url, "getRelayStatus", json!({"relayId":prepared["id"]}))
        .await
        .unwrap();
    assert_eq!(refused["error"]["code"], -32111);
    let database = format!(
        "sqlite://{}?mode=rwc",
        directory.join("relay.sqlite").display()
    );
    let observation = Store::open(&database, 1).await.unwrap();
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let row = observation
                .connection()
                .query_one(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    "SELECT state FROM relay_relays".to_owned(),
                ))
                .await
                .unwrap()
                .unwrap();
            let stored: String = row.try_get("", "state").unwrap();
            let state: Value = serde_json::from_str(&stored).unwrap();
            if state["state"]["status"] == "confirmed" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("original confirmation");
    // Give the tracker another pass to prove it does not resend a finalized wire.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        std::fs::read_to_string(directory.join("send-calls")).unwrap(),
        "send\n"
    );
    durable(
        &directory.join("result.json"),
        serde_json::to_string_pretty(&json!({"window":window,
        "transport":"real-loopback-http", "storage":"real-sqlite", "crash":"SIGKILL",
        "chain":"fake", "proofs":"fake", "exactOriginalRecovered":true, "sendCount":1,
        "wireSha256":hex::encode(Sha256::digest(&wire.0)), "status":"PASS"}))
        .unwrap()
        .as_bytes(),
    );
    restarted.kill_and_wait();
}

#[tokio::test]
async fn http_prepared_wire_survives_sigkill_without_automatic_commit() {
    crash_case("prepared").await;
}
#[tokio::test]
async fn http_sent_marker_survives_sigkill_before_network() {
    crash_case("sent-before-network").await;
}
#[tokio::test]
async fn http_lost_response_survives_sigkill_after_network() {
    crash_case("after-network-before-response").await;
}
