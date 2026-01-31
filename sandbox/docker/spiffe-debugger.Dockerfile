ARG SPIRE_VERSION=1.14.0

FROM --platform=$TARGETPLATFORM rust:1.85-alpine AS rust-builder
WORKDIR /work
COPY . .
RUN apk add --no-cache clang lld musl-dev protobuf-dev \
    && cargo build -p spire-agent --release

FROM --platform=$TARGETPLATFORM ghcr.io/spiffe/spire-agent:${SPIRE_VERSION} AS spire-agent-go
FROM --platform=$TARGETPLATFORM ghcr.io/spiffe/spire-server:${SPIRE_VERSION} AS spire-server-go

FROM --platform=$TARGETPLATFORM alpine:3.20
RUN apk add --no-cache ca-certificates openssl curl

COPY --from=spire-agent-go /opt/spire/bin/spire-agent /usr/local/bin/spire-agent-go
COPY --from=spire-server-go /opt/spire/bin/spire-server /usr/local/bin/spire-server-go
COPY --from=rust-builder /work/target/release/spire-agent /usr/local/bin/spire-agent-rust

CMD ["sleep", "3600"]
