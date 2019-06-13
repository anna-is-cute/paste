FROM debian:stretch

RUN apt-get update
RUN apt-get install \
  --no-install-recommends \
  --assume-yes \
  curl ca-certificates \
  git \
  build-essential \
  cmake \
  autoconf automake libtool \
  libssl1.0-dev libssh-dev libz-dev clang \
  libpq-dev \
  pkg-config

WORKDIR /

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-05-10

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc

WORKDIR /paste
