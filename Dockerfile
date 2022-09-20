FROM rust:latest AS builder

WORKDIR /usr/src/tado-exporter

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev musl-tools && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.* .
COPY src/ ./src

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
LABEL name="tado-exporter"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/src/tado-exporter/target/x86_64-unknown-linux-musl/release/tado-exporter /

CMD ["/tado-exporter"]