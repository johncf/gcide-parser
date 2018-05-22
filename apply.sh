#!/bin/bash

set -e

pushd "$(dirname "$0")" >/dev/null
cargo build --release
for f in $(ls gcide/CIDE.*); do ./target/release/identity $f; done
popd >/dev/null
