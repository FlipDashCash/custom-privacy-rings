# Ring devnet stack

`tools/ring-devnet-deploy.sh` runs the ring services for devnet on ECS, and `just ring-devnet <command>` calls it. Every AWS name starts with `zolana-ring-devnet`, and every resource carries the tag `zolana-ring-devnet=1`. The stack builds its images into its own ECR repositories and shares no repository, distribution, database, secret or KMS key with another stack.

## Services

- **Photon** indexes devnet into its own Postgres instance. A one-shot task runs `photon-migration up` on each `up`, because Photon migrates only its own SQLite.
- **Ring indexer** keeps its index in its own Postgres instance and binds `0.0.0.0:8786`. It serves the ring methods and passes every other call to Photon. Clients read Photon through it.
- **Ring RPC task** runs two containers. The Ring RPC serves port 8785. The ring prover binds `127.0.0.1:3001` beside it, because the Rust prover client accepts plain HTTP only on loopback.
- **Ring RPC settings** follow from the task. It runs in derived mode with its store in a third Postgres instance. Its custody manifest pins a storage authority, a fee payer and a co-signer and holds a wrapped grant rows secret. They turn on the record writer, the relay, co-signing and `database` reader grants.
- **Enrollment writer** writes the enrollments of a ring after the ring registers through `registerEnrollmentRing`. The ring's policy authority signs the registration, and the ring's `enrollmentWriter` role must name the writer key. The Postgres store keeps the registered rings across a restart. `RING_DEVNET_ENROLLMENT_BUDGET_LAMPORTS` caps its batch fees, 1 SOL by default. `RING_DEVNET_ENROLLMENT_RING_BUDGET_LAMPORTS` caps the fees of one ring, 0.02 SOL by default, and `RING_DEVNET_ENROLLMENT_MAX_RINGS` caps the registered rings, 64 by default.
- **Network** is one load balancer in front of the three services. One CloudFront distribution per service serves HTTPS on an Amazon hostname.

## Prerequisites

- The aws CLI v2 with write access to ECS, ECR, EC2, load balancing, RDS, CloudFront, IAM, Secrets Manager, KMS and CloudWatch Logs.
- `iam:PassRole` on `zolana-ring-devnet-exec` and `zolana-ring-devnet-ring-rpc-task`.
- The KMS actions `CreateKey`, `TagResource`, `CreateAlias`, `DescribeKey`, `GetPublicKey` and `GenerateDataKeyWithoutPlaintext`. `down --purge` also needs `ListResourceTags`, `DeleteAlias` and `ScheduleKeyDeletion`.
- `docker` with `buildx`, `jq`, `openssl`, `python3`, `git`, `curl` and the `zolana` submodule checked out.
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

   `up` builds each missing image, creates or updates every resource, waits for the services and prints the `status` block. The first `up` also creates the KMS keys and the custody manifest.
4. Fund the storage authority, the fee payer and the enrollment writer that `status` prints. The storage authority pays every records write. The fee payer pays every relay, and the relay budget caps what enrollments and token accounts take from it. The enrollment writer pays every enrollment batch, and the enrollment budget caps it.
5. Run `just ring-devnet status` until each service reads `ready` and `ring rpc keys` reads `match custody`.
6. Copy the `demo env` lines of `status` into the devnet profile of the demo.
7. Start the loopback prover that `RING_PROVER_URL` names:

   ```bash
   RING_DEVNET_INDEXER_URL=<ring indexer URL> just ring-devnet-prover
   ```

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

storage authority  <base58>   (fund it, it pays every records write)
fee payer          <base58>   (fund it above the relay budget)
enrollment writer  <base58>   (fund it above the enrollment budget)

demo env, RING_PROVER_URL is the loopback prover of just ring-devnet-prover
RING_SOLANA_RPC_URL=https://api.devnet.solana.com
RING_INDEXER_URL=https://<id>.cloudfront.net
RING_PROVER_URL=http://127.0.0.1:3001
RING_RPC_URL=https://<id>.cloudfront.net
RING_GENESIS_HASH=EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG
RING_RPC_SERVICE_KEY=<base58>
RING_RPC_COSIGNER=<base58>
RING_ENROLLMENT_WRITER=<base58>
RING_STORAGE_PROGRAM_ID=rnGSTosTQ9gToNQP44THEZph8xtBtuGa5FEwzhKrWjL

photon             ready
ring indexer       ready
ring rpc           ready
ring rpc keys      match custody
logs               aws logs tail /zolana-ring-devnet --region eu-north-1 --follow
```

Each address comes from the custody manifest. The `demo env` lines follow the keys of `tools/demo-env.md`. `ring rpc keys` compares the public keys of the Ring RPC `health` method with the manifest. `differ from custody` means the task started from another manifest.

## Images

`up` publishes each missing image through `zolana/tools/publish-image.sh` into `zolana-ring-devnet-ring-rpc`, `zolana-ring-devnet-ring-indexer`, `zolana-ring-devnet-ring-prover` and `zolana-ring-devnet-photon`. The ring tags name the commit, and the Photon tag names the `zolana` pin. `RING_DEVNET_RING_RPC_TAG`, `RING_DEVNET_RING_INDEXER_TAG`, `RING_DEVNET_PROVER_TAG` and `RING_DEVNET_PHOTON_TAG` pick another tag.

`prover/Dockerfile` builds the prover from `prover/cmd/prover`. The image bakes every release key of `prover/ring/proving-keys.lock` and the SPP keys `transfer_ring_1_2.key` and `transfer_ring_2_2.key`, each checked against its pinned sha256.

## Custody

The Ring RPC signs through AWS KMS. The first `up` creates six KMS keys under the aliases `alias/zolana-ring-devnet/ring-rpc/<name>`:

- `service`, `storage-authority`, `co-signer`, `fee-payer` and `enrollment-applier` are `ECC_NIST_EDWARDS25519` signing keys. The enrollment applier is the `enrollmentWriter` role key each ring names.
- `wrapping` is a symmetric key. It wraps the derived mode auditor root and the grant rows secret.

`up` then writes the custody manifest to the secret `zolana-ring-devnet/ring-rpc/custody`. The manifest pins the ARN and the address of each signing key and holds each wrapped secret as KMS ciphertext. It holds no secret in plaintext. The task reads it as `RING_RPC_CUSTODY_JSON`, beside `RING_RPC_DEPLOYMENT=zolana-ring-devnet` and `AWS_REGION`.

Each wrapped secret binds the KMS encryption context `application=zolana-custom-rings-v1`, `deployment=zolana-ring-devnet`, the devnet genesis hash and its purpose. The Ring RPC opens a secret only under the same context.

A later `up` reuses the stored manifest and creates no key. It checks each alias against the manifest before it changes a role, a secret or a service. A missing alias, an alias that names another key or a key that is not enabled stops it. A new auditor root changes every derived auditor key and orphans every ring, and a new service key breaks every client pin.

The task role `zolana-ring-devnet-ring-rpc-task` holds one inline policy, `custody`:

- `kms:GetPublicKey` and `kms:Sign` on the five signing keys. `kms:Sign` takes only `ED25519_SHA_512` over a raw message.
- `kms:Decrypt` on the wrapping key, only under the stack encryption context with the purpose `auditor-root` or `grant-rows`.

Only an ECS task of the account in the stack region assumes the task role.

`zolana-ring-devnet/solana-rpc-url` holds the RPC URL the services read. `RING_DEVNET_RPC_URL` creates or replaces it, and the value never reaches a command line or the dry run. The task execution role reads only the custody manifest, the RPC URL and the three database passwords. Any other secret under the same prefix stays out of its reach.

## Browser origins

The Ring RPC takes one WebAuthn RP id. It refuses to start unless each allowed origin host is that id or, for a domain id, a subdomain of it. `up` applies the same check before it deploys.

The default allows `http://localhost:3000` under the id `localhost`, the origin of `next dev` and the live specs. The hosted demo takes its own origin and id:

```bash
RING_DEVNET_ALLOW_ORIGINS=https://privacy.helius.xyz RING_DEVNET_WEBAUTHN_RP_ID=privacy.helius.xyz just ring-devnet up
```

## Teardown

`down` deletes the services, task definitions, cluster, distributions, load balancer, target groups, databases, security groups, log group, database passwords, task role and execution role. It keeps the KMS keys, the custody manifest, the RPC URL secret, the ECR repositories and a final snapshot of the Ring RPC store. A later `up` then serves every ring with the same keys.

`down --purge` also deletes the custody manifest, the RPC URL secret and the ECR repositories, and takes no final snapshot. A snapshot from an earlier `down` stays. It removes the alias of each KMS key and schedules the key for deletion after 7 days. It acts only on a key that a stack alias names and that carries the stack tag. It deletes each secret whose name starts with `zolana-ring-devnet/` and that carries the stack tag.

`down` deletes a distribution only when its Comment names a service of the stack and it carries the stack tag. It never touches `E7DAYCSYTCIWK`, the distribution zolnet-devnet serves its Photon and prover on.

## Limits

- The stack holds no storage program keypair. The upstream key holder deploys the storage program at `STORAGE_PROGRAM_ID`, `rnGSTosTQ9gToNQP44THEZph8xtBtuGa5FEwzhKrWjL`, on devnet. Until then the Ring RPC record writer has no program to write to.
- ECS gives the task role to each container of a task, so the prover can also sign and open secrets through KMS.
- The stack serves devnet alone, because each wrapped secret binds the devnet genesis hash.
- Each service runs one task. A deploy stops the old task before the new one starts, because the Ring RPC store and each index take one writer.
- The Ring RPC serves either `localhost` or a hosted origin, never both. An `up` with the other origin and id switches it.
- Clients reach the Ring RPC from load balancer addresses and share its rate buckets. The task raises each client rate to ten times the Ring RPC default.
- The load balancer also answers plain HTTP on ports 8784, 8785 and 8786. Browsers use the HTTPS URLs of `status`.
- A set `RING_DEVNET_PHOTON_START_SLOT` applies at every Photon start. Leave it unset on a later `up`, and Photon resumes from its last indexed slot.
- The ring indexer refuses a stored index that started at another slot. Keep a numeric `RING_DEVNET_RING_INDEXER_START_SLOT` for the life of the index. `latest` always passes.

## Pitfalls

- The public devnet RPC throttles a Photon backfill. Set `RING_DEVNET_RPC_URL` to a provider URL before the first `up`.
- A custody manifest deleted by hand makes the next `up` wrap a new auditor root, and every ring loses its auditor. Copy the secret value before any change to it.
