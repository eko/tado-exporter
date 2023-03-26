FROM rust:latest AS builder

WORKDIR /usr/src/tado-exporter

ARG TARGETOS
ARG TARGETARCH
ARG TARGETVARIANT
ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN echo "I'm building for $TARGETOS/$TARGETARCH/$TARGETVARIANT"
RUN echo "I am running on $BUILDPLATFORM, building for $TARGETPLATFORM"

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev musl-tools gcc-arm* && \
    rm -rf /var/lib/apt/lists/*


COPY scripts/ ./

#RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add armv7-unknown-linux-gnueabihf
#RUN ./setup.sh $TARGETARCH

COPY Cargo.* .
COPY src/ ./src

#RUN cargo build --target x86_64-unknown-linux-musl --release
RUN cargo build --target armv7-unknown-linux-gnueabihf --release
#RUN build.sh $TARGETARCH

FROM scratch
LABEL name="tado-exporter"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/tado-exporter /

CMD ["/tado-exporter"]