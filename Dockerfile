FROM --platform=$BUILDPLATFORM rust:slim-bullseye AS builder

WORKDIR /usr/src/tado-exporter

ARG TARGETOS
ARG TARGETARCH
ARG TARGETVARIANT
ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN echo "I'm building for $TARGETOS/$TARGETARCH/$TARGETVARIANT"
RUN echo "I am running on $BUILDPLATFORM, building for $TARGETPLATFORM"

RUN apt update && \
    # apt-get -y install ca-certificates libssl-dev musl-tools gcc-arm* libfindbin-libs-perl perl make && \
    apt install -y ca-certificates libssl-dev g++-arm-linux-gnueabihf libc6-dev-armhf-cross gcc-arm* libfindbin-libs-perl perl make && \
    rm -rf /var/lib/apt/lists/*


COPY scripts/ ./

#RUN rustup target add x86_64-unknown-linux-musl
RUN rustup toolchain install stable
RUN rustup target add armv7-unknown-linux-gnueabihf
#RUN ./setup.sh $TARGETARCH

COPY Cargo.* .
COPY src/ ./src

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
 
#RUN cargo build --target x86_64-unknown-linux-musl --release
RUN cargo build --target armv7-unknown-linux-gnueabihf --release
#RUN build.sh $TARGETARCH

RUN ls .
RUN ls /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/
RUN ls /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/tado-exporter

#FROM scratch
FROM --platform=$BUILDPLATFORM alpine:latest
LABEL name="tado-exporter"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/src/tado-exporter/target/armv7-unknown-linux-gnueabihf/release/tado-exporter /


CMD ["/tado-exporter"]