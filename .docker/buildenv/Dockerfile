FROM debian:stretch

RUN apt-get update && apt-get install \
  --no-install-recommends \
  --assume-yes \
  curl ca-certificates \
  git \
  build-essential \
  cmake \
  autoconf automake libtool \
  libssl-dev libz-dev clang \
  libpq-dev \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-11-25

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc

WORKDIR /paste
