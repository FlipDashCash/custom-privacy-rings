#!/usr/bin/env bash
# Loopback ring prover for devnet on port 3001 that reads the key registry from the hosted ring indexer.
set -euo pipefail
: "${RING_DEVNET_INDEXER_URL:?set RING_DEVNET_INDEXER_URL to the hosted ring indexer}"
cd "$(dirname "${BASH_SOURCE[0]}")/.."

just_bin="${JUST:-just}"
port=3001
url="http://127.0.0.1:$port"
workdir=target/ring-devnet
log="$workdir/prover.log"

"$just_bin" check-custom-ring-keys
keys_dir="$("$just_bin" --evaluate spp-keys-dir)"
mkdir -p "$workdir"
lsof -ti "tcp:$port" -sTCP:LISTEN 2>/dev/null | xargs kill -9 2>/dev/null || true
sleep 1
PROVER_INDEXER_URL="$RING_DEVNET_INDEXER_URL" nohup target/prover-server start --keys-dir "$keys_dir" \
    --prover-address "127.0.0.1:$port" --metrics-address 127.0.0.1:9998 --auto-download=true \
    ${ZOLANA_PROVER_REDIS_URL:+--redis-url "$ZOLANA_PROVER_REDIS_URL"} \
    > "$log" 2>&1 &
printf 'waiting for prover %s ' "$url"
for _ in $(seq 1 120); do
    if curl -sf --max-time 5 "$url/health" >/dev/null 2>&1; then
        echo " ready"
        echo "  prover    $url  (indexer $RING_DEVNET_INDEXER_URL, log $log)"
        echo "stop with 'just ring-devnet-services-stop'"
        exit 0
    fi
    printf '.'
    sleep 1
done
echo " timed out"
echo "the prover did not become ready, see $log" >&2
exit 1
