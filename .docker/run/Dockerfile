FROM jkcclemens/paste

RUN $HOME/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

RUN apt-get install --assume-yes --no-install-recommends postgresql-client

ADD run.sh /run.sh

CMD /run.sh
