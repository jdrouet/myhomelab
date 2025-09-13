FROM --platform=$BUILDPLATFORM rust:1-alpine AS vendor

ENV USER=root

WORKDIR /code

RUN cargo init --name myhomelab --vcs none

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml
