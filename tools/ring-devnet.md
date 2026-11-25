# Ring devnet stack

`tools/ring-devnet-deploy.sh` runs the ring services for devnet on ECS, and `just ring-devnet <command>` calls it. Every AWS name starts with `zolana-ring-devnet`, and every resource carries the tag `zolana-ring-devnet=1`. The stack builds its images into its own ECR repositories and shares no repository, distribution, database or secret with another stack.

## Services

- **Photon** indexes devnet into its own Postgres instance. A one-shot task runs `photon-migration up` on each `up`, because Photon migrates only its own SQLite.
- **Ring indexer** keeps its index in its own Postgres instance and binds `0.0.0.0:8786`. It serves the ring methods and passes every other call to Photon. Clients read Photon through it.
- **Ring RPC task** runs two containers. The Ring RPC serves port 8785. The ring prover binds `127.0.0.1:3001` beside it, because the Rust prover client accepts plain HTTP only on loopback.
- **Ring RPC settings** follow from the task. It runs in derived mode with its store in a third Postgres instance. Its storage authority, fee payer, co-signer and grant rows keys turn on the record writer, the relay, co-signing and `database` reader grants.
- **Enrollment writer** writes the enrollments of a ring after the ring registers through `registerEnrollmentRing`. The ring's policy authority signs the registration, and the ring's `enrollmentWriter` role must name the writer key. The Postgres store keeps the registered rings across a restart. `RING_DEVNET_ENROLLMENT_BUDGET_LAMPORTS` caps its batch fees, 1 SOL by default. `RING_DEVNET_ENROLLMENT_RING_BUDGET_LAMPORTS` caps the fees of one ring, 0.02 SOL by default, and `RING_DEVNET_ENROLLMENT_MAX_RINGS` caps the registered rings, 64 by default.
- **Network** is one load balancer in front of the three services. One CloudFront distribution per service serves HTTPS on an Amazon hostname.

## Prerequisites

- The aws CLI v2 with write access to ECS, ECR, EC2, load balancing, RDS, CloudFront, IAM, Secrets Manager and CloudWatch Logs.
- `docker` with `buildx`, `jq`, `openssl`, `python3`, `git` and the `zolana` submodule checked out.
- A devnet RPC URL with block and transaction history. The Ring RPC reads relay status from transaction history.

## Steps

1. Commit the tree. `zolana/tools/publish-image.sh` refuses a dirty tree, and each image tag names the commit.
2. Preview every AWS call and image build. The dry run runs none and changes nothing:

   ```bash
   RING_DEVNET_PHOTON_START_SLOT=<slot> tools/ring-devnet-deploy.sh up --dry-run
   ```

3. Bring the stack up. The start slot must be at or before the creation of the devnet SPP trees, or Photon misses their earlier leaves:

   ```bash
   RING_DEVNET_PHOTON_START_SLOT=<slot> RING_DEVNET_RPC_URL=<devnet RPC URL> just ring-devnet up
   ```

   `up` builds each missing image, creates or updates every resource, waits for the services and prints the `status` block.
4. Fund the storage authority, the fee payer and the enrollment writer that `status` prints. The storage authority pays every records write. The fee payer pays every relay, and the relay budget caps what enrollments and token accounts take from it. The enrollment writer pays every enrollment batch, and the enrollment budget caps it.
5. Copy the endpoint block into the devnet profile of the demo.

## Status

`just ring-devnet status` prints the endpoint block:

```text
solana rpc         https://api.devnet.solana.com
photon             https://<id>.cloudfront.net
ring indexer       https://<id>.cloudfront.net
ring rpc           https://<id>.cloudfront.net
service key        <base58>
co-signer key      <base58>
enrollment writer  <base58>
```

The demo reads the enrollment writer as `RING_ENROLLMENT_WRITER`. `status` then prints the storage authority, the fee payer and the enrollment writer to fund, one readiness line per service and the log command. Each key comes from the `public-key` tag of its secret, so `status` reads no secret value.

## Images

`up` publishes each missing image through `zolana/tools/publish-image.sh` into `zolana-ring-devnet-ring-rpc`, `zolana-ring-devnet-ring-indexer`, `zolana-ring-devnet-ring-prover` and `zolana-ring-devnet-photon`. The ring tags name the commit, and the Photon tag names the `zolana` pin. `RING_DEVNET_RING_RPC_TAG`, `RING_DEVNET_RING_INDEXER_TAG`, `RING_DEVNET_PROVER_TAG` and `RING_DEVNET_PHOTON_TAG` pick another tag.

`prover/Dockerfile` builds the prover from `prover/cmd/prover`. The image bakes every release key of `prover/ring/proving-keys.lock` and the SPP keys `transfer_ring_1_2.key` and `transfer_ring_2_2.key`, each checked against its pinned sha256.

## Keys

`up` creates each key once under `zolana-ring-devnet/ring-rpc/` and never rotates it:

- `root-secret` is the derived mode root. A new root changes every derived auditor key and orphans every ring.
- `service-key` is the key every client pins.
- `storage-authority-key`, `fee-payer-key` and `co-signer-key` are Ed25519 seeds.
- `grant-rows-key` keys the reader grant rows.

`zolana-ring-devnet/enrollment-writer-keypair` holds the Ed25519 seed of the enrollment writer, the `enrollmentWriter` role key each ring names. `up` creates it once in the same way.

Each value is 64 lowercase hex characters with no newline. To bring an existing key, create its secret before the first `up`, and `up` adds its `public-key` tag.

`zolana-ring-devnet/solana-rpc-url` holds the RPC URL the services read. `RING_DEVNET_RPC_URL` creates or replaces it, and the value never reaches a command line or the dry run. The task execution role reads only `ring-rpc/`, `photon/`, `ring-indexer/`, `enrollment-writer-keypair` and `solana-rpc-url` under the prefix. Any other secret under the same prefix stays out of its reach.

## Browser origins

The Ring RPC takes one WebAuthn RP id. It refuses to start unless each allowed origin host is that id or, for a domain id, a subdomain of it. `up` applies the same check before it deploys.

The default allows `http://localhost:3000` under the id `localhost`, the origin of `next dev` and the live specs. The hosted demo takes its own origin and id:

```bash
RING_DEVNET_ALLOW_ORIGINS=https://privacy.helius.xyz RING_DEVNET_WEBAUTHN_RP_ID=privacy.helius.xyz just ring-devnet up
```

## Teardown

`down` deletes the services, task definitions, cluster, distributions, load balancer, target groups, databases, security groups, log group, database passwords and execution role. It keeps the key secrets, the RPC URL secret, the ECR repositories and a final snapshot of the Ring RPC store. A later `up` then serves every ring with the same keys. `down --purge` deletes those as well and takes no final snapshot.

`down` deletes a distribution only when its Comment names a service of the stack and it carries the stack tag. It never touches `E7DAYCSYTCIWK`, the distribution zolnet-devnet serves its Photon and prover on.

## Limits

- The stack holds no storage program keypair. The upstream key holder deploys the storage program at `STORAGE_PROGRAM_ID`, `rnGSTosTQ9gToNQP44THEZph8xtBtuGa5FEwzhKrWjL`, on devnet. Until then the Ring RPC record writer has no program to write to.
- Each service runs one task. A deploy stops the old task before the new one starts, because the Ring RPC store and each index take one writer.
- The Ring RPC serves either `localhost` or a hosted origin, never both. An `up` with the other origin and id switches it.
- Clients reach the Ring RPC from load balancer addresses and share its rate buckets. The task raises each client rate to ten times the Ring RPC default.
- The load balancer also answers plain HTTP on ports 8784, 8785 and 8786. Browsers use the HTTPS URLs of `status`.
- A set `RING_DEVNET_PHOTON_START_SLOT` applies at every Photon start. Leave it unset on a later `up`, and Photon resumes from its last indexed slot.
- The ring indexer refuses a stored index that started at another slot. Keep a numeric `RING_DEVNET_RING_INDEXER_START_SLOT` for the life of the index. `latest` always passes.

## Pitfalls

- The public devnet RPC throttles a Photon backfill. Set `RING_DEVNET_RPC_URL` to a provider URL before the first `up`.
- A secret created by hand from a Solana JSON keypair stops the Ring RPC at start. Store the 32-byte seed as hex.
