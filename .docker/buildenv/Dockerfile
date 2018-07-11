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

RUN git clone -b stable git://github.com/jedisct1/libsodium.git /libsodium

WORKDIR /libsodium

RUN ./autogen.sh
RUN ./configure --prefix="$HOME/libsodium-build"
RUN make install

WORKDIR /

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-06-28

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc
RUN echo "export PKG_CONFIG_PATH=$HOME/libsodium-build/lib/pkgconfig" >> $HOME/.bashrc
RUN echo "export LD_LIBRARY_PATH=$HOME/libsodium-build/lib" >> $HOME/.bashrc

WORKDIR /paste
