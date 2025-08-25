FROM rust:1-bookworm

RUN apt-get update \
  && apt-get upgrade -y \
  && apt-get install -y dropbear libdbus-1-dev pkg-config \
  && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:$PATH"

RUN echo 'export RUSTUP_HOME="/usr/local/rustup"' >> /root/.profile
RUN echo 'export CARGO_HOME="/usr/local/cargo"' >> /root/.profile
RUN echo 'export PATH="/usr/local/cargo/bin:${PATH}"' >> /root/.profile

RUN rustup default stable \
  && rustup component add rust-analyzer \
  && rustup component add rustfmt clippy \
  && cargo -V && rustc -V

WORKDIR /code

CMD ["/usr/sbin/dropbear", "-p", "0.0.0.0:2222", "-F", "-E", "-B"]

