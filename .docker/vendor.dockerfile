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

RUN cargo init --lib --name myhomelab-adapter-opentelemetry adapter/opentelemetry
COPY adapter/opentelemetry/Cargo.toml /code/adapter/opentelemetry/Cargo.toml

RUN cargo init --lib --name myhomelab-adapter-sqlite adapter/sqlite
COPY adapter/sqlite/Cargo.toml /code/adapter/sqlite/Cargo.toml

RUN cargo init --lib --name myhomelab-sensor-manager sensor/manager
COPY sensor/manager/Cargo.toml /code/sensor/manager/Cargo.toml

RUN cargo init --lib --name myhomelab-sensor-prelude sensor/prelude
COPY sensor/prelude/Cargo.toml /code/sensor/prelude/Cargo.toml

RUN cargo init --lib --name myhomelab-sensor-system sensor/system
COPY sensor/system/Cargo.toml /code/sensor/system/Cargo.toml

RUN cargo init --lib --name myhomelab-sensor-xiaomi-lywsd03mmc-atc sensor/xiaomi/lywsd03mmc-atc
COPY sensor/xiaomi/lywsd03mmc-atc/Cargo.toml /code/sensor/xiaomi/lywsd03mmc-atc/Cargo.toml

RUN cargo init --lib --name myhomelab-sensor-xiaomi-miflora sensor/xiaomi/miflora
COPY sensor/xiaomi/miflora/Cargo.toml /code/sensor/xiaomi/miflora/Cargo.toml

RUN cargo init --lib --name myhomelab-client-web client/web
COPY client/web/Cargo.toml /code/client/web/Cargo.toml

RUN cargo init --lib --name myhomelab-dashboard domain/dashboard
COPY domain/dashboard/Cargo.toml /code/domain/dashboard/Cargo.toml

RUN cargo init --lib --name myhomelab-event domain/event
COPY domain/event/Cargo.toml /code/domain/event/Cargo.toml

RUN cargo init --lib --name myhomelab-metric domain/metric
COPY domain/metric/Cargo.toml /code/domain/metric/Cargo.toml

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
