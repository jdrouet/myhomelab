# syntax = devthefuture/dockerfile-x

FROM ./.docker/vendor AS vendor

FROM rust:1-bookworm AS base

RUN apt-get update \
    && apt-get install -y dbus libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
COPY LICENSE /code/LICENSE
COPY src /code/src
COPY systemd /code/systemd

FROM base AS build-deb-package

RUN cargo install cargo-deb
RUN cargo deb

FROM scratch AS deb-package

COPY --from=build-deb-package /code/target/debian /
