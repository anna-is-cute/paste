FROM ubuntu:19.10

RUN apt-get update && apt-get install \
  --no-install-recommends \
  --assume-yes \
  curl ca-certificates \
  git \
  build-essential \
  cmake \
  autoconf automake libtool \
  libssl-dev libssh-dev libz-dev clang \
  libpq-dev \
  pkg-config
RUN apt-get autoremove --assume-yes
RUN apt-get clean autoclean
RUN rm -rf /var/lib/{apt,dpkg,cache,log}/

WORKDIR /

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-05-10

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc

WORKDIR /paste
