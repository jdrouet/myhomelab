# syntax = devthefuture/dockerfile-x

FROM ./.docker/debian AS builder

RUN cargo build --release --package myhomelab-sensor-xiaomi-miflora

FROM scratch AS binary

# can be used to build the binary and output it on the host
# docker build -t myhomelab-sensor-xiaomi-miflora -f sensor/xiaomi/miflora/debian.Dockerfile --target binary --output=type=local,dest=target/docker .
COPY --from=builder /code/target/release/myhomelab-sensor-xiaomi-miflora /myhomelab-sensor-xiaomi-miflora

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y dbus \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /code/target/release/myhomelab-sensor-xiaomi-miflora /usr/bin/myhomelab-sensor-xiaomi-miflora

ENV RUST_LOG=debug

CMD ["/usr/bin/myhomelab-sensor-xiaomi-miflora"]
