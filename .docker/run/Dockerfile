FROM jkcclemens/paste

RUN $HOME/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

RUN apt-get update
RUN apt-get install --assume-yes --no-install-recommends postgresql-client

STOPSIGNAL SIGKILL

ADD run.sh /run.sh

CMD /run.sh
