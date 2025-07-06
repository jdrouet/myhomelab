# syntax = devthefuture/dockerfile-x

FROM ./.docker/alpine AS builder

RUN cargo build --release --package myhomelab-server

FROM alpine:3

RUN apk add --no-cache dbus-libs bluez-libs

COPY --from=builder /code/target/release/myhomelab-server /usr/bin/myhomelab-server

ENV MYHOMELAB_HTTP_HOST=0.0.0.0
ENV MYHOMELAB_HTTP_PORT=3000
ENV MYHOMELAB_DATASET_PATH=/data/config.toml
ENV MYHOMELAB_SQLITE_PATH=/data/myhomelab.db
ENV RUST_LOG=info,tower_http=debug

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/myhomelab-server"]
