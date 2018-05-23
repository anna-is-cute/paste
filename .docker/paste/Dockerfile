FROM rustlang/rust:nightly

WORKDIR /app

ENV PASTE_CONFIG_PATH paste.toml

RUN apt update -y && apt install -y libsodium-dev cmake build-essential postgresql postgresql-client

# TODO: get it from .env instead? but then it's not accessable by the container.
COPY postgres_env /root/postgres_env
RUN sed -E 's/[^=]+=//g' /root/postgres_env | tr "\n" " " | awk '{ print "db:5432:" $1 ":" $2 ":" $3 }' > /root/.pgpass && chmod 0600 /root/.pgpass && rm /root/postgres_env

RUN cargo install diesel_cli --no-default-features --features postgres

RUN diesel migration run --migration-dir="/app/migrations" --database-url="postgres://paste:paste@db/paste"
RUN cargo build

CMD ["cargo", "run", "--release", "${PASTE_CONFIG_PATH}"]
