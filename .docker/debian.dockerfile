# syntax = devthefuture/dockerfile-x

FROM ./vendor AS vendor

FROM rust:1-bookworm AS base

RUN apt-get update \
  && apt-get install -y dbus libdbus-1-dev \
  && rm -rf /var/lib/apt/lists/*

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY adapter/dataset /code/adapter/dataset
COPY adapter/http/client /code/adapter/http/client
COPY adapter/http/server /code/adapter/http/server
COPY adapter/http/shared /code/adapter/http/shared
COPY adapter/opentelemetry /code/adapter/opentelemetry
COPY adapter/sqlite /code/adapter/sqlite
COPY client/web /code/client/web
COPY domain/dashboard /code/domain/dashboard
COPY domain/event /code/domain/event
COPY domain/metric /code/domain/metric
COPY sensor/manager /code/sensor/manager
COPY sensor/prelude /code/sensor/prelude
COPY sensor/system /code/sensor/system
COPY sensor/xiaomi/lywsd03mmc-atc /code/sensor/xiaomi/lywsd03mmc-atc
COPY sensor/xiaomi/miflora /code/sensor/xiaomi/miflora
COPY prelude /code/prelude
COPY server /code/server
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
