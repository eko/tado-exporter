#!/bin/bash

if [ $# -eq 0 ]
  then
    echo "No arguments supplied: ARCH must be specified"
    exit 1
fi

TARGET=$(./evaluate-target.sh $1)

cargo build --target $TARGET --release