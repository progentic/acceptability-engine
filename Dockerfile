FROM rust:1-bookworm AS builder

ARG CARGO_BUILD_JOBS=1
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
ENV CARGO_INCREMENTAL=0

WORKDIR /src
COPY core ./core
WORKDIR /src/core
RUN cargo build --release

FROM rust:1-bookworm AS runtime

ARG CARGO_BUILD_JOBS=1
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
ENV CARGO_INCREMENTAL=0

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl git build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy
RUN cargo install cargo-audit --locked
RUN cargo install cargo-deny --locked

RUN useradd --create-home --shell /usr/sbin/nologin engine \
    && mkdir -p /data /artifacts /workspaces \
    && chown -R engine:engine /data /artifacts /workspaces

COPY --from=builder /src/core/target/release/core /usr/local/bin/acceptability-engine

USER engine
ENV AH_WORKSPACE_MODE=local
ENV AH_SANDBOX_PROFILE=development
ENV RUST_LOG=core=info
EXPOSE 8080
VOLUME ["/data", "/artifacts", "/workspaces"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail http://127.0.0.1:8080/health/ready || exit 1

ENTRYPOINT ["/usr/local/bin/acceptability-engine"]
CMD ["--workspace", "/workspaces", "--database", "/data/evidence.db", "--artifact-root", "/artifacts", "--port", "8080"]
