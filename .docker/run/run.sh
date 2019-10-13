#!/bin/bash -l

set -e

source "$HOME/.bashrc"

while ! pg_isready -h postgres -p 5432 -q; do
  sleep 1
done

export CARGO_TARGET_DIR="/paste/.docker/run/target"

diesel migration --migration-dir=webserver/migrations run

cargo build -p worker_email "$@"
cargo build -p worker_delete_directory "$@"
cargo build -p worker_expire_paste "$@"

cargo run "$@" -p webserver config.toml
