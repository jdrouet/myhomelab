# syntax = devthefuture/dockerfile-x

FROM ./vendor AS vendor

FROM rust:1-alpine AS base

RUN apk add --no-cache \
    build-base \
    dbus-dev \
    pkgconf \
    bluez-dev \
    musl-dev

ENV RUSTFLAGS="-C target-feature=-crt-static"
ENV PKG_CONFIG_ALL_STATIC=1

WORKDIR /code

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY adapter-file /code/adapter-file
COPY adapter-http/client /code/adapter-http/client
COPY adapter-http/server /code/adapter-http/server
COPY adapter-http/shared /code/adapter-http/shared
COPY adapter-sqlite /code/adapter-sqlite
COPY agent/core /code/agent/core
COPY agent/prelude /code/agent/prelude
COPY agent/reader-system /code/agent/reader-system
COPY agent/reader-xiaomi/lywsd03mmc-atc /code/agent/reader-xiaomi/lywsd03mmc-atc
COPY client/tui /code/client/tui
COPY dashboard /code/dashboard
COPY metric /code/metric
COPY prelude /code/prelude
COPY server /code/server
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
