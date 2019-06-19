FROM ubuntu:19.10

RUN apt-get update
RUN apt-get install \
  --no-install-recommends \
  --assume-yes \
  curl ca-certificates \
  git \
  build-essential \
  cmake \
  autoconf automake libtool \
  libssl-dev libssh-dev libz-dev clang \
  libpq-dev \
  pkg-config \
  ruby libgmp-dev
RUN apt-get autoremove --assume-yes
RUN apt-get clean autoclean
RUN rm -rf /var/lib/{apt,dpkg,cache,log}/

RUN ln -s /usr/lib/x86_64-linux-gnu/libruby-2.5.so{.2.5,}
RUN gem install rouge

WORKDIR /

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-05-10

RUN echo "source $HOME/.cargo/env" >> $HOME/.bashrc

WORKDIR /paste
