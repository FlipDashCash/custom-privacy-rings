# The ring prover of prover/cmd/prover, built from the repository root with the
# keys of prover/ring/proving-keys.lock and the SPP keys of SPP_KEYS baked in.
# Each key is checked against its pinned sha256 here and again when it loads.

FROM golang:1.27.3-alpine@sha256:d200d2dd76b03a99b52651fd1325aff7dcd35422c35c01bac56571b0507f3deb AS builder

WORKDIR /src
COPY zolana/prover/server/go.mod zolana/prover/server/go.sum zolana/prover/server/
COPY prover/go.mod prover/go.sum prover/
RUN cd prover && go mod download && go mod verify

COPY zolana/prover/server zolana/prover/server
COPY prover prover
RUN cd prover && sh ../zolana/prover/server/build-release.sh /usr/local/bin/light-prover ./cmd/prover \
    && light-prover release-keys > /tmp/release-keys \
    && test -s /tmp/release-keys \
    && mkdir -p /tmp/empty_proving_keys

FROM alpine:3.22@sha256:5291449c3df73caf6ed85e649dec1b9e818b39a5d8c871e97afc13e9cd5e8fa8 AS keys

ARG SPP_KEYS="transfer_ring_1_2.key transfer_ring_2_2.key"
ARG SPP_KEYS_URL=https://d3gbdb0egjwcw9.cloudfront.net

RUN apk add --no-cache curl=8.14.1-r4
COPY --from=builder /tmp/release-keys /tmp/release-keys
COPY zolana/prover/server/prover/provingkeys/proving-keys.lock /tmp/spp-keys.lock
# The SPP lockfile prints one field per line, the line after a key name holds its sha256.
RUN set -eu; \
    prefix="$(awk '$1 == "\"prefix\":" { gsub(/[",]/, "", $2); print $2 }' /tmp/spp-keys.lock)"; \
    test -n "$prefix"; \
    for name in $SPP_KEYS; do \
        sha256="$(awk -v key="\"$name\":" '$1 == key { getline; gsub(/[",]/, "", $2); print $2; exit }' /tmp/spp-keys.lock)"; \
        test -n "$sha256" || { echo "$name has no pin in the SPP lockfile" >&2; exit 1; }; \
        echo "$name $sha256 $SPP_KEYS_URL/$prefix/$name" >> /tmp/release-keys; \
    done; \
    mkdir /keys; \
    while read -r name sha256 url; do \
        curl -fsSL --retry 3 --connect-timeout 10 -o "/keys/$name" "$url"; \
        echo "$sha256  /keys/$name" | sha256sum -c -; \
    done < /tmp/release-keys; \
    touch -d @0 /keys /keys/*

FROM gcr.io/distroless/base-debian11:nonroot@sha256:68b0f492c1eb077f71d384b544456dac4afb282372966917df32b3923f65ad0d

# Below the binary layer, writable for any other SPP key the prover fetches.
COPY --chown=nonroot:nonroot --from=keys /keys/ /proving-keys/
COPY --chown=nonroot:nonroot --from=builder /tmp/empty_proving_keys /proving-keys/
COPY --from=builder /usr/local/bin/light-prover /usr/local/bin/light-prover

LABEL org.opencontainers.image.title="Zolana Ring Prover" \
      org.opencontainers.image.description="Zolana Groth16 proving server with the custom ring circuits and keys" \
      org.opencontainers.image.source="https://github.com/helius-labs/zolana-custom-rings" \
      org.opencontainers.image.licenses="Apache-2.0"

WORKDIR /

# 3001 prover API, 9998 Prometheus metrics.
EXPOSE 3001 9998

ENTRYPOINT ["light-prover"]
CMD ["start", "--require-optimized-build", "--keys-dir", "/proving-keys/"]
