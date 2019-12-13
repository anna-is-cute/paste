FROM jkcclemens/paste

RUN $HOME/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

FROM debian:stable-slim

COPY --from=0 /root/.cargo/bin/diesel /usr/local/bin/diesel

RUN apt-get update && apt-get install \
  --no-install-recommends \
  --assume-yes \
  ca-certificates \
  openssl libpq5 \
  postgresql-client \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get clean

STOPSIGNAL SIGKILL

WORKDIR /paste

ADD . .

CMD /paste/run.sh
