#!/bin/bash

set -e

source "$HOME/.bashrc"

while ! pg_isready -h postgres -p 5432 -q; do
  sleep 1
done

diesel migration --migration-dir=webserver/migrations run

cargo build -p worker_email "$@"
cargo build -p worker_delete_all_pastes "$@"

cargo run "$@" -p webserver config.toml
