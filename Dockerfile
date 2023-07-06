
FROM --platform=$BUILDPLATFORM rust:slim-bullseye AS builder

RUN apt update && \
    apt install -y ca-certificates libssl-dev libfindbin-libs-perl make && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/tado-exporter

COPY scripts/ ./
COPY Cargo.* .
COPY src/ ./src
RUN rustup toolchain install stable

# Work only on AMD64, NO CROSS COMPILE Tested on Windows
FROM builder as builder-amd64
ENV TARGET=x86_64-unknown-linux-gnu

# Work only on ARM64, NO CROSS COMPILE Tested OSX
FROM builder as builder-arm64
ENV TARGET=aarch64-unknown-linux-gnu

FROM builder as builder-armv7
ENV TARGET=armv7-unknown-linux-gnueabihf
RUN apt update && \
    apt install -y libc6-dev-armhf-cross gcc-arm-linux-gnueabihf && \
    rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++

FROM builder-$TARGETARCH$TARGETVARIANT as final-builder
RUN rustup target add ${TARGET}
RUN cargo build --target ${TARGET} --release --target-dir /build && \
    cp /build/$TARGET/release/tado-exporter / && \
    rm -rf /build

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
    if [ "$TARGETARCH$TARGETVARIANT" -eq "armv7"]; then apt install patchelf; fi && \
    rm -rf /var/lib/apt/lists/*

COPY --from=final-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=final-builder /tado-exporter /usr/bin/


RUN if [ "$TARGETARCH$TARGETVARIANT" -eq "armv7"]; then patchelf --set-interpreter /lib/ld-linux-armhf.so.3 /tado-exporter; fi

CMD ["tado-exporter"]