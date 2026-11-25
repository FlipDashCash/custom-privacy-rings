#!/usr/bin/env bash
# Keys and a background serve of a derived mode Ring RPC on a local stack.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ring_rpc="${RING_RPC_BIN:-$root/target/debug/ring-rpc}"

usage() {
    cat >&2 <<'USAGE'
tools/ring-rpc-local.sh keys DIR SOLANA_RPC_URL [fee-payer] [co-signer] [enrollment-writer]
tools/ring-rpc-local.sh serve DIR PORT LOG [ring-rpc serve flag...]
tools/ring-rpc-local.sh pubkey KEY_FILE
RING_RPC_BIN names the ring-rpc binary, target/debug/ring-rpc by default.
USAGE
    exit 2
}

keygen() {
    "$ring_rpc" keygen --kind "$1" --out "$2"
}

keys() {
    local dir="$1" rpc_url="$2" kind payer
    shift 2
    mkdir -p "$dir"
    keygen root "$dir/root.secret" >/dev/null
    keygen service "$dir/service.key" >/dev/null
    keygen grant-rows "$dir/grant-rows.key" >/dev/null
    # The record writer pays every records write from the storage authority.
    payer="$(keygen storage-authority "$dir/storage-authority.key" | sed -n 's/^storage authority *//p')"
    solana airdrop 100 "$payer" --url "$rpc_url" >/dev/null
    for kind in "$@"; do
        case "$kind" in
            fee-payer)
                payer="$(keygen fee-payer "$dir/fee-payer.secret" | sed -n 's/^fee payer *//p')"
                solana airdrop 100 "$payer" --url "$rpc_url" >/dev/null
                ;;
            co-signer) keygen co-signer "$dir/co-signer.key" >/dev/null ;;
            # The ring's enrollmentWriter role key signs and pays each enrollment batch.
            enrollment-writer)
                payer="$(keygen enrollment-applier "$dir/enrollment-writer.key" | sed -n 's/^enrollment applier *//p')"
                solana airdrop 100 "$payer" --url "$rpc_url" >/dev/null
                ;;
            *) usage ;;
        esac
    done
}

serve() {
    local dir="$1" port="$2" log="$3" url="http://127.0.0.1:$2"
    shift 3
    local args=(
        --port "$port"
        --root-secret-file "$dir/root.secret" --service-key-file "$dir/service.key"
        --storage-authority-key-file "$dir/storage-authority.key"
        --grant-rows-key-file "$dir/grant-rows.key"
        --webauthn-rp-id localhost
    )
    [[ ! -f "$dir/fee-payer.secret" ]] || args+=(--fee-payer-key-file "$dir/fee-payer.secret")
    # An empty approve set refuses every operation.
    [[ ! -f "$dir/co-signer.key" ]] ||
        args+=(--co-signer-key-file "$dir/co-signer.key"
            --co-signer-approve "outflow,withdrawal,transfers,deposits,withdrawals")
    # A zero budget refuses every enrollment write, the caller names the rings.
    [[ ! -f "$dir/enrollment-writer.key" ]] ||
        args+=(--enrollment-applier-key-file "$dir/enrollment-writer.key"
            --enrollment-budget-lamports 1000000000)
    held() { lsof -t -i "tcp:$port" -s tcp:listen 2>/dev/null; }
    held | xargs kill 2>/dev/null || true
    for _ in $(seq 1 50); do
        held >/dev/null || break
        sleep 0.2
    done
    if held >/dev/null; then
        echo "port $port is still held" >&2
        exit 1
    fi
    nohup "$ring_rpc" serve "${args[@]}" "$@" > "$log" 2>&1 &
    printf 'waiting for ring rpc %s ' "$url"
    for _ in $(seq 1 60); do
        if curl -sf --max-time 5 "$url/health" >/dev/null 2>&1; then
            echo " ready"
            return 0
        fi
        printf '.'
        sleep 1
    done
    echo " timed out"
    echo "the ring rpc did not answer, see $log" >&2
    exit 1
}

# The key files hold the lowercase hex of the Ed25519 seed.
pubkey() {
    # shellcheck disable=SC2016
    node -e '
const { createPrivateKey, createPublicKey } = require("node:crypto");
const seed = Buffer.from(require("node:fs").readFileSync(process.argv[1], "utf8").trim(), "hex");
if (seed.length !== 32) throw new Error(`${process.argv[1]} holds no 32-byte seed`);
const pkcs8 = Buffer.concat([Buffer.from("302e020100300506032b657004220420", "hex"), seed]);
const spki = createPublicKey(createPrivateKey({ key: pkcs8, format: "der", type: "pkcs8" }))
  .export({ format: "der", type: "spki" }).subarray(-32);
const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
let n = BigInt(`0x${spki.toString("hex")}`), out = "";
for (; n > 0n; n /= 58n) out = alphabet[Number(n % 58n)] + out;
for (const byte of spki) { if (byte) break; out = `1${out}`; }
console.log(out);
' "$1"
}

case "${1:-}" in
    keys) (($# >= 3)) || usage; shift; keys "$@" ;;
    serve) (($# >= 4)) || usage; shift; serve "$@" ;;
    pubkey) (($# == 2)) || usage; pubkey "$2" ;;
    *) usage ;;
esac
