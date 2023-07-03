FROM --platform=$BUILDPLATFORM rust:slim-bullseye AS builder

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
RUN echo "I am running on $BUILDPLATFORM, building for $TARGETPLATFORM"
ENV BUILDER_NAME=builder-$TARGETARCH-$TARGETVARIANT
RUN echo "$BUILDER_NAME and builder-$TARGETARCH-$TARGETVARIANT"

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

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM builder as builder-arm-v7
RUN rustup target add armv7-unknown-linux-gnueabihf

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++

RUN cargo build --target armv7-unknown-linux-gnueabihf --release

FROM --platform=$TARGETPLATFORM debian:bullseye-slim
LABEL name="tado-exporter"

RUN apt update && \
    apt install patchelf


RUN echo "EXPORTER ${BUILDER_NAME}"

COPY --from=builder-arm-v7 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder-arm-v7 /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/tado-exporter /

# ENV LD_LIBRARY_PATH="/usr/arm-linux-gnueabihf/lib:${LD_LIBRARY_PATH}"
RUN patchelf --set-interpreter /lib/ld-linux-armhf.so.3 /tado-exporter

CMD ["/tado-exporter"]