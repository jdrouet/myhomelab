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
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
