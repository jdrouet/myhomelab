# syntax = devthefuture/dockerfile-x

FROM ./base AS server-builder

RUN cargo build --release --package myhomelab-server

FROM debian:bookworm-slim

COPY --from=server-builder /code/target/release/myhomelab-server /usr/bin/myhomelab-server

ENV HOST=0.0.0.0
ENV PORT=3000
ENV MYHOMELAB_DATASET_PATH=/data/config.toml
ENV MYHOMELAB_SQLITE_PATH=/data/myhomelab.db
ENV RUST_LOG=info,tower_http=debug

VOLUME ["/data"]

ENTRYPOINT ["/usr/bin/myhomelab-server"]
