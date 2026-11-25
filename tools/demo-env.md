# `demo.env` contract

`tools/demo-localnet.sh` writes `demo.env` for the localnet it starts. The demo reads the file through `RING_LOCALNET_ENV_FILE`. Each key below takes one unquoted `KEY=value` line:

- `RING_SOLANA_RPC_URL`, the Solana JSON-RPC URL of the validator.
- `RING_INDEXER_URL`, the URL of the ring indexer, which sits in front of Photon and serves the ring key registry.
- `RING_PROVER_URL`, the URL of the prover for SPP and ring proofs.
- `RING_RPC_URL`, the base URL of the Ring RPC.
- `RING_GENESIS_HASH`, the base58 genesis hash of the cluster, which every signed read and record update binds.
- `RING_RPC_SERVICE_KEY`, the base58 public key of the Ring RPC service key, which the client pins and every signed read binds.
- `RING_RPC_COSIGNER`, the base58 public key the Ring RPC co-signs with under the asset limits.
- `RING_ENROLLMENT_WRITER`, the base58 public key the Ring RPC signs and pays enrollment batches with. A ring that names it as its `enrollmentWriter` role gets its enrollments written on chain after its policy authority registers the ring through `registerEnrollmentRing`.
- `RING_STORAGE_PROGRAM_ID`, the base58 program id of the storage program on the cluster.
- `RING_LOCALNET_AUTHORITY_SEED`, the lowercase hex of the 32-byte seed of the localnet SPP protocol authority, which signs ring activation.
- `RING_TEST_MINT`, the base58 SPL token mint that `tools/demo-localnet.sh` creates for the demo.

## Localnet authority seed

The seed is public. It is the ASCII bytes of `zolana localnet snapshot authori`, the governance seed of `sdk-libs/ts/test/e2e/ring-live-helpers.ts`. The localnet account snapshots name its address as the protocol authority:

```text
RING_LOCALNET_AUTHORITY_SEED=7a6f6c616e61206c6f63616c6e657420736e617073686f7420617574686f7269
address HF3DijLymE9gVYeG3kX1N19WZXu5BH5yWBgsvnVtm7tw
```
