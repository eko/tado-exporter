
FROM --platform=$BUILDPLATFORM rust:slim-bullseye AS builder

RUN apt update && \
    apt install -y ca-certificates libssl-dev libc6-dev-armhf-cross gcc-arm-linux-gnueabihf libfindbin-libs-perl make patchelf && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/tado-exporter

COPY scripts/ ./
COPY Cargo.* .
COPY src/ ./src
RUN rustup toolchain install stable

FROM builder as builder-amd64
RUN rustup target add x86_64-unknown-linux-musl
ENV TARGET=x86_64-unknown-linux-musl


FROM builder as builder-armv7
RUN rustup target add armv7-unknown-linux-gnueabihf

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
ENV TARGET=armv7-unknown-linux-gnueabihf


FROM builder-$TARGETARCH$TARGETVARIANT as final-builder

RUN cargo build --target ${TARGET} --release

FROM --platform=$TARGETPLATFORM debian:bullseye-slim
LABEL name="tado-exporter"

ARG TARGETOS
ARG TARGETARCH
ARG TARGETVARIANT
ARG TARGETPLATFORM
ARG BUILDOS
ARG BUILDARCH
ARG BUILDVARIANT
ARG BUILDPLATFORM

RUN echo "I'm building for $TARGETOS/$TARGETARCH/$TARGETVARIANT"
RUN echo "I'm building on $BUILDOS/$BUILDARCH/$BUILDVARIANT"

RUN echo "builder-$TARGETARCH$TARGETVARIANT"

RUN apt update && \
    apt install patchelf

COPY --from=final-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=final-builder /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/tado-exporter /

RUN patchelf --set-interpreter /lib/ld-linux-armhf.so.3 /tado-exporter

CMD ["/tado-exporter"]