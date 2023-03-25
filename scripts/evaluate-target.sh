#!/bin/bash

if [ $# -eq 0 ]
  then
    echo "No arguments supplied: ARCH must be specified"
    exit 1
fi

ARCH=$1
EVN=
if [ "$ARCH" = "amd64" ]
then
    ENV="x86_64"
elif [ "$ARCH" = "armv7" ]
then
    ENV=armv7
else
    echo "Wrong ARCH. Not Supported"
    exit 2
fi

TARGET=$ENV-unknown-linux-musl
echo $TARGET