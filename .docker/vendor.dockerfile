FROM --platform=$BUILDPLATFORM rust:1-alpine AS vendor

ENV USER=root

WORKDIR /code

RUN cargo init

RUN cargo init --lib --name myhomelab-adapter-dataset adapter/dataset
COPY adapter/dataset/Cargo.toml /code/adapter/dataset/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-http-client adapter/http/client
COPY adapter/http/client/Cargo.toml /code/adapter/http/client/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-http-server adapter/http/server
COPY adapter/http/server/Cargo.toml /code/adapter/http/server/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-http-shared adapter/http/shared
COPY adapter/http/shared/Cargo.toml /code/adapter/http/shared/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-sqlite adapter/sqlite
COPY adapter/sqlite/Cargo.toml /code/adapter/sqlite/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-manager agent/manager
COPY agent/manager/Cargo.toml /code/agent/manager/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-prelude agent/prelude
COPY agent/prelude/Cargo.toml /code/agent/prelude/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-sensor-system agent/system
COPY agent/system/Cargo.toml /code/agent/system/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-sensor-xiaomi-lywsd03mmc-atc agent/xiaomi/lywsd03mmc-atc
COPY agent/xiaomi/lywsd03mmc-atc/Cargo.toml /code/agent/xiaomi/lywsd03mmc-atc/Cargo.toml

RUN cargo init --lib --name myhomelab-agent-sensor-xiaomi-miflora agent/xiaomi/miflora
COPY agent/xiaomi/miflora/Cargo.toml /code/agent/xiaomi/miflora/Cargo.toml

RUN cargo init --lib --name myhomelab-client-web client/web
COPY client/web/Cargo.toml /code/client/web/Cargo.toml

RUN cargo init --lib --name myhomelab-dashboard dashboard
COPY dashboard/Cargo.toml /code/dashboard/Cargo.toml

RUN cargo init --lib --name myhomelab-event event
COPY event/Cargo.toml /code/event/Cargo.toml

RUN cargo init --lib --name myhomelab-metric metric
COPY metric/Cargo.toml /code/metric/Cargo.toml

RUN cargo init --lib --name myhomelab-prelude prelude
COPY prelude/Cargo.toml /code/prelude/Cargo.toml

RUN cargo init --lib --name myhomelab-server server
COPY server/Cargo.toml /code/server/Cargo.toml

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml
