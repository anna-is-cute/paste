#!/bin/bash -l

set -e

mkdir -p /libraries
rm -rf /libraries/*
cp exec/shasums /libraries/shasums
cp exec/*.so /libraries/

source "$HOME/.bashrc"

while ! pg_isready -h postgres -p 5432 -q; do
  sleep 1
done

diesel migration --migration-dir=webserver/migrations run

./exec/webserver config.toml
