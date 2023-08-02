#!/bin/bash

curl https://sh.rustup.rs -sSf | sh -s -- --profile minimal --default-toolchain nightly -y
source "$HOME/.cargo/env"
pip3 install maturin
maturin build --compatibility manylinux2014 --profile=release --out dist