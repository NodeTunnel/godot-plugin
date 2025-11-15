#!/bin/bash
cd src
cargo build --release
cp target/release/libnodetunnel.so ../addons/nodetunnel/bin/