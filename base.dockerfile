FROM --platform=$BUILDPLATFORM rust:1-bookworm AS vendor

ENV USER=root

WORKDIR /code

RUN cargo init
COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock

RUN cargo init --lib --name myhomelab-adapter-http-client adapter-http/client
COPY adapter-http/client/Cargo.toml /code/adapter-http/client/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-http-server adapter-http/server
COPY adapter-http/server/Cargo.toml /code/adapter-http/server/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-http-shared adapter-http/shared
COPY adapter-http/shared/Cargo.toml /code/adapter-http/shared/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-sqlite adapter-sqlite
COPY adapter-sqlite/Cargo.toml /code/adapter-sqlite/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-core agent/core
COPY agent/core/Cargo.toml /code/agent/core/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-prelude agent/prelude
COPY agent/prelude/Cargo.toml /code/agent/prelude/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-reader-system agent/reader-system
COPY agent/reader-system/Cargo.toml /code/agent/reader-system/Cargo.toml

RUN cargo init --lib --name myhomelab-metric metric
COPY metric/Cargo.toml /code/metric/Cargo.toml

RUN cargo init --lib --name myhomelab-metric-mock metric-mock
COPY metric-mock/Cargo.toml /code/metric-mock/Cargo.toml

RUN cargo init --lib --name myhomelab-prelude prelude
COPY prelude/Cargo.toml /code/prelude/Cargo.toml

RUN cargo init --lib --name myhomelab-server server
COPY server/Cargo.toml /code/server/Cargo.toml


# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml

FROM rust:1-bookworm AS base

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY adapter-http/client /code/adapter-http/client
COPY adapter-http/server /code/adapter-http/server
COPY adapter-http/shared /code/adapter-http/shared
COPY adapter-sqlite /code/adapter-sqlite
COPY agent/core /code/agent/core
COPY agent/prelude /code/agent/prelude
COPY agent/reader-system /code/agent/reader-system
COPY metric /code/metric
COPY metric-mock /code/metric-mock
COPY prelude /code/prelude
COPY server /code/server
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
