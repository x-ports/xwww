#!/bin/sh

# This is a helper script to use just before releasing a new version
# (it helps us not forget anything, as has happened before)

if [ $# -lt 1 ]; then
	echo "Usage: $0 <new version name, as 'MAJOR.MINOR.PATCH'>"
	exit 1
fi

pkill xwww-daemon

set -e

typos

# don't forget updating dependencies and testing everything
cargo update
cargo build
cargo test --workspace -- --include-ignored
./doc/gen.sh # make sure the docs "compile"

# Cargo.toml:
sed -e "s/^version = .*/version = \"$1\"/" Cargo.toml > TMP
mv -v TMP Cargo.toml

# Make sure it still builds (just to be 100% sure), and to update Cargo.lock
cargo build
