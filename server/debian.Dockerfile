# syntax = devthefuture/dockerfile-x

FROM ./.docker/debian AS server-builder

RUN cargo build --release --package myhomelab-server

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y dbus \
    && rm -rf /var/lib/apt/lists/*

COPY --from=server-builder /code/target/release/myhomelab-server /usr/bin/myhomelab-server

ENV MYHOMELAB_HTTP_HOST=0.0.0.0
ENV MYHOMELAB_HTTP_PORT=3000
ENV MYHOMELAB_DATASET_PATH=/data/config.toml
ENV MYHOMELAB_SQLITE_PATH=/data/myhomelab.db
ENV RUST_LOG=info,tower_http=debug

VOLUME ["/data"]

CMD ["/usr/bin/myhomelab-server"]
