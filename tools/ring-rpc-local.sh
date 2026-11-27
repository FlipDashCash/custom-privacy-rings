#!/usr/bin/env bash
# The local KMS, custody and a background serve of a derived mode Ring RPC on a local stack.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export RING_RPC_BIN="${RING_RPC_BIN:-$root/target/debug/ring-rpc}"
export RING_RPC_DEPLOYMENT="${RING_RPC_DEPLOYMENT:-localnet}"
custody="$root/tools/localnet-custody.sh"

usage() {
    cat >&2 <<'USAGE'
tools/ring-rpc-local.sh kms PORT LOG
tools/ring-rpc-local.sh custody DIR SOLANA_RPC_URL [co-signer] [enrollment-writer]
tools/ring-rpc-local.sh serve DIR PORT LOG [ring-rpc serve flag...]
tools/ring-rpc-local.sh address DIR ROLE
kms prints the endpoint that custody and serve read from RING_RPC_LOCALNET_KMS.
RING_RPC_BIN names the ring-rpc binary, target/debug/ring-rpc by default.
USAGE
    exit 2
}

free_port() {
    local port="$1" only="${2:-}" pid
    held() { lsof -t -i "tcp:$port" -s tcp:listen 2>/dev/null; }
    for pid in $(held); do
        if [[ -n "$only" ]] && ! ps -o command= -p "$pid" | grep -q "$only"; then
            echo "port $port is held by another process, move it with ZOLANA_LOCALNET_KMS_PORT" >&2
            exit 1
        fi
        kill "$pid" 2>/dev/null || true
    done
    for _ in $(seq 1 50); do
        held >/dev/null || return 0
        sleep 0.2
    done
    echo "port $port is still held" >&2
    exit 1
}

# The KMS holds every key in memory, so a restart orphans the custody it served.
kms() {
    local port="$1" log="$2" endpoint=""
    # Only a local KMS of an earlier run gives up the port.
    free_port "$port" 'localnet-kms\.mjs'
    (ulimit -c 0 && exec nohup node "$root/tools/localnet-kms.mjs" "$port") >"$log" 2>&1 &
    for _ in $(seq 1 50); do
        IFS= read -r endpoint <"$log" || true
        [[ -z "$endpoint" ]] || break
        sleep 0.2
    done
    if [[ ! "$endpoint" =~ ^127\.0\.0\.1:[0-9]+$ ]]; then
        echo "the local KMS did not listen, see $log" >&2
        exit 1
    fi
    echo "$endpoint"
}

custody() {
    local dir="$1" rpc_url="$2" genesis writer=null co_signer=null kind role payee
    shift 2
    : "${RING_RPC_LOCALNET_KMS:?run tools/ring-rpc-local.sh kms first}"
    for kind in "$@"; do
        case "$kind" in
            co-signer) co_signer="$("$custody" signer)" ;;
            # The ring's enrollmentWriter role key signs and pays each enrollment batch.
            enrollment-writer) writer="$("$custody" signer)" ;;
            *) usage ;;
        esac
    done
    genesis="$(solana genesis-hash --url "$rpc_url")"
    mkdir -p "$dir"
    "$custody" rpc "$genesis" derived "$writer" relay |
        jq --argjson co_signer "$co_signer" '. + {$co_signer}' >"$dir/rpc-custody.json"
    # The record writer pays from the storage authority, the relay from the fee payer.
    for role in storage_authority fee_payer enrollment_applier; do
        payee="$(jq -r --arg role "$role" '.[$role].address // empty' "$dir/rpc-custody.json")"
        [[ -z "$payee" ]] || solana airdrop 100 "$payee" --url "$rpc_url" >/dev/null
    done
}

serve() {
    local dir="$1" port="$2" log="$3" url="http://127.0.0.1:$2" manifest="$1/rpc-custody.json"
    shift 3
    : "${RING_RPC_LOCALNET_KMS:?run tools/ring-rpc-local.sh kms first}"
    local args=(
        --port "$port" --custody "$manifest" --localnet-kms "$RING_RPC_LOCALNET_KMS"
        --webauthn-rp-id localhost
    )
    # An empty approve set refuses every operation.
    ! jq -e '.co_signer != null' "$manifest" >/dev/null ||
        args+=(--co-signer-approve "outflow,withdrawal,transfers,deposits,withdrawals")
    # A zero budget refuses every enrollment write, the caller names the rings.
    ! jq -e '.enrollment_applier != null' "$manifest" >/dev/null ||
        args+=(--enrollment-budget-lamports 1000000000)
    free_port "$port"
    nohup "$RING_RPC_BIN" serve "${args[@]}" "$@" > "$log" 2>&1 &
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

case "${1:-}" in
    kms) (($# == 3)) || usage; kms "$2" "$3" ;;
    custody) (($# >= 3)) || usage; shift; custody "$@" ;;
    serve) (($# >= 4)) || usage; shift; serve "$@" ;;
    address) (($# == 3)) || usage; jq -er --arg role "$3" '.[$role].address' "$2/rpc-custody.json" ;;
    *) usage ;;
esac
