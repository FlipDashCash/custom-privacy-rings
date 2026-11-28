# Measured Ring Deposit on Localnet

Run this opt-in test against a disposable ledger with transaction history and
Solana transaction v1 enabled. The test funds execution accounts in the same
transaction as settlement. It checks finalized vault credit, the tree, and the
one-use SPP address reservation. It then opens the SPP events with the test auditor
key and withdraws SOL, classic SPL and both fixture Token-2022 assets from a fresh
wallet using independently replayed providers.

## Build and start the fixture

1. Initialize the Git submodules, install npm dependencies, and build the
   `@heliuslabs/spr` workspace. Use the build and setup commands in the
   `_test-ts-live` recipe in the root `justfile` for the version-matched
   Custom Ring, stock SPP, registry, storage, Photon, Ring Indexer, CLIs, and
   prover. Serve the pinned SPP `transfer_confidential_1_2.key` for native address
   reservations.
2. Start a fresh Localnet with the generated protocol account snapshots and
   those program binaries, including canonical SPP tree 0 with address-slot
   capacity. Use Agave with transaction history enabled for
   testing v1 RPC submission and finalized reads. Use loopback endpoints for
   the validator, primary Photon, Ring Indexer, prover, Ring RPC and the
   test-only KMS in `tools/localnet-kms-start.sh`.
3. Use the recipe's CLI initialization with `mode = "anonymous"` in
   `ring.toml` and an empty policy. On this ledger the Ring must be active,
   its records registry pristine, and its enrollment writer funded. Enable
   enrollments for the fixture Ring. Use single-Ring custody with the
   `records` profile of `tools/localnet-custody.sh`. The test pays network
   fees with its own validator-funded keys.
4. Create a new validator-funded keypair and the test mints below. Use a
   dedicated Solana config file with the same RPC URL and the new payer
   keypair before running the commands.

```sh
# SOLANA_CONFIG is the path to the dedicated Localnet Solana config file.
classic=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
token22=TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
legacy_token() { spl-token --config "$SOLANA_CONFIG" --program-id "$classic" "$@"; }
token_2022() { spl-token --config "$SOLANA_CONFIG" --program-id "$token22" "$@"; }

# Copy the returned addresses into the variables used by the later commands.
legacy_token create-token --decimals 0
token_2022 create-token --decimals 0 --enable-metadata
token_2022 create-token --decimals 0 --transfer-fee-basis-points 250 --transfer-fee-maximum-fee 100
token_2022 create-token --decimals 0 --transfer-fee-basis-points 0 --transfer-fee-maximum-fee 100
token_2022 initialize-metadata "$METADATA_MINT" "Measured fixture" MEASURED https://example.invalid/metadata
legacy_token create-account "$CLASSIC_MINT"
legacy_token mint "$CLASSIC_MINT" 100000
for mint in "$METADATA_MINT" "$FEE_MINT" "$ZERO_FEE_MINT"; do
  token_2022 create-account "$mint"
  token_2022 mint "$mint" 100000
done

jq -n --arg classic "$CLASSIC_MINT" --arg metadata "$METADATA_MINT" \
  --arg fee "$FEE_MINT" --arg zeroFee "$ZERO_FEE_MINT" \
  '{classic:$classic,metadata:$metadata,fee:$fee,zeroFee:$zeroFee}' > assets.json
```

## Configure and run the test

Set `RING_MEASURED_CONTEXT` to the absolute path of a local JSON fixture.
Keep the fixture and all local key files outside the source tree.

| JSON field                                                  | Value                                                                                                                 |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `rpcUrl`, `indexerUrl`, `proverUrl`, `ringRpcUrl`, `kmsUrl` | Primary HTTP loopback origins                                                                                         |
| `genesisHash`, `tree`, `ringProgram`                        | The fresh validator's genesis hash, snapshot tree and active Ring program                                             |
| `payerFile`, `assetFile`, `custodyFile`                     | Absolute paths to the validator-funded payer JSON keypair, `assets.json`, and the local KMS Ring RPC custody manifest |
| `closedServices`                                            | Primary Photon, Ring Indexer, prover, Ring RPC and test KMS origins that the switch will stop                         |
| `switchCommand`                                             | Absolute path to an executable that switches to independent providers                                                 |
| `independentNetworkFile`                                    | Absolute path where the switch writes the independent RPC, indexer and prover origins                                 |

Have the switch command stop the fixture's primary services by their process
IDs and leave the validator running with its ledger. Start stock Photon and
Ring Indexer on different ports, with fresh databases and `--start-slot`
set to the first available block on the fixture ledger. Point the indexer
at the new Photon endpoint. Start a fresh prover pointed at the new indexer,
with the same public proving keys. Wait for the services to be ready and
Photon to catch up to the finalized slot at the switch. Write a JSON
object with `rpcUrl`, `indexerUrl` and `proverUrl` and return. The project
recipes provide the primary launch arguments. Use `/readiness`,
`/ready` and `/health` for Photon, the indexer and the prover respectively.

```sh
cd sdk-libs/ts
RING_MEASURED_CONTEXT=/absolute/path/to/context.json \
  npx vitest run test/e2e/ring-measured-output.live.test.ts
```

The test reports finalized settlement and withdrawal signatures, exact amounts
in atomic units, signed sizes, compute units and fees. Stop the fixture's
remaining processes after the run.
