FROM --platform=$BUILDPLATFORM rust:1-alpine AS vendor

ENV USER=root

WORKDIR /code

RUN cargo init

RUN cargo init --lib --name myhomelab-adapter-dataset adapter/dataset --vcs none \
  && cargo init --lib --name myhomelab-adapter-http-client adapter/http/client --vcs none \
  && cargo init --lib --name myhomelab-adapter-http-server adapter/http/server --vcs none \
  && cargo init --lib --name myhomelab-adapter-http-shared adapter/http/shared --vcs none \
  && cargo init --lib --name myhomelab-adapter-opentelemetry adapter/opentelemetry --vcs none \
  && cargo init --lib --name myhomelab-adapter-sqlite adapter/sqlite --vcs none \
  && cargo init --lib --name myhomelab-sensor-manager sensor/manager --vcs none \
  && cargo init --lib --name myhomelab-sensor-prelude sensor/prelude --vcs none \
  && cargo init --lib --name myhomelab-sensor-system sensor/system --vcs none \
  && cargo init --lib --name myhomelab-sensor-xiaomi-lywsd03mmc-atc sensor/xiaomi/lywsd03mmc-atc --vcs none \
  && cargo init --lib --name myhomelab-sensor-xiaomi-miflora sensor/xiaomi/miflora --vcs none \
  && cargo init --lib --name myhomelab-client-web client/web --vcs none \
  && cargo init --lib --name myhomelab-dashboard domain/dashboard --vcs none \
  && cargo init --lib --name myhomelab-event domain/event --vcs none \
  && cargo init --lib --name myhomelab-metric domain/metric --vcs none \
  && cargo init --lib --name myhomelab-prelude prelude --vcs none \
  && cargo init --lib --name myhomelab-server server --vcs none

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY adapter/dataset/Cargo.toml /code/adapter/dataset/Cargo.toml
COPY adapter/http/client/Cargo.toml /code/adapter/http/client/Cargo.toml
COPY adapter/http/server/Cargo.toml /code/adapter/http/server/Cargo.toml
COPY adapter/http/shared/Cargo.toml /code/adapter/http/shared/Cargo.toml
COPY adapter/opentelemetry/Cargo.toml /code/adapter/opentelemetry/Cargo.toml
COPY adapter/sqlite/Cargo.toml /code/adapter/sqlite/Cargo.toml
COPY sensor/manager/Cargo.toml /code/sensor/manager/Cargo.toml
COPY sensor/prelude/Cargo.toml /code/sensor/prelude/Cargo.toml
COPY sensor/system/Cargo.toml /code/sensor/system/Cargo.toml
COPY sensor/xiaomi/lywsd03mmc-atc/Cargo.toml /code/sensor/xiaomi/lywsd03mmc-atc/Cargo.toml
COPY sensor/xiaomi/miflora/Cargo.toml /code/sensor/xiaomi/miflora/Cargo.toml
COPY client/web/Cargo.toml /code/client/web/Cargo.toml
COPY domain/dashboard/Cargo.toml /code/domain/dashboard/Cargo.toml
COPY domain/event/Cargo.toml /code/domain/event/Cargo.toml
COPY domain/metric/Cargo.toml /code/domain/metric/Cargo.toml
COPY prelude/Cargo.toml /code/prelude/Cargo.toml
COPY server/Cargo.toml /code/server/Cargo.toml

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
  --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
  mkdir -p /code/.cargo \
  && cargo vendor >> /code/.cargo/config.toml
