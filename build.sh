#!/bin/bash
set -e
cd src
cargo build --release

case "$(uname -s)" in
  Darwin) LIB=libnodetunnel.dylib ;;
  *)      LIB=libnodetunnel.so ;;
esac

cp "target/release/$LIB" ../addons/nodetunnel/bin/
echo "copied $LIB -> addons/nodetunnel/bin/"
