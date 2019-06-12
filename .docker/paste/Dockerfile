FROM jkcclemens/paste

RUN $HOME/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

RUN apt-get update && apt-get install --assume-yes --no-install-recommends postgresql-client

STOPSIGNAL SIGKILL

WORKDIR /paste

ADD . .

CMD /paste/run.sh
