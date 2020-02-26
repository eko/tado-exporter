#!/bin/bash
set -euo pipefail

rustup target add $TARGET

if [ "$TARGET" = "arm-unknown-linux-gnueabihf" ]
then
  git clone --depth=1 https://github.com/raspberrypi/tools.git /tmp/tools
  export PATH=/tmp/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH
fi

cargo build --target=$TARGET --release

tar -C target/$TARGET/release -czf mybinary-$TRAVIS_TAG-$TARGET.tar.gz tado_exporter