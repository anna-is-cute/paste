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
  && rm -rf /var/lib/apt/lists/* \
  && apt-get clean

WORKDIR /

RUN curl --proto 'https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly-2020-05-15

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc

WORKDIR /paste
