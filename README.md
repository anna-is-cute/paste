# paste

*A sensible pastebin.*

## Unfinished: some features do not work

See the issues.

## Idea

Pretty much every pastebin sucks. With Gist removing anonymous pastes, the search for a new pastebin
began, and then it shortly ended. There's not much out there.

There should be a pastebin that's easy-to-use and simple, supporting multiple files, syntax
highlighting, anonymity, and secure authentication.

## Goals

- [ ] API
  - [ ] Sensible
  - [ ] Sane
  - [ ] Simple
  - [ ] Secure
- [ ] Front-end
  - [ ] Pretty
  - [ ] Performant
  - [ ] Functional

## Usage

- Clone the repo (`--recursive` for submodules)
- Create a postgres database and user
- `echo 'DATABASE_URL=postgres://username@/database' > .env`
- `cargo install diesel_cli --no-default-features --features postgres`
- `diesel migration run`
- Edit `config.toml`
- `cargo run --release path/to/config.toml`
- Preferably use `ROCKET_ENV=prod` and set a secret key in `Rocket.toml`
  - See [Rocket docs](https://rocket.rs/guide/configuration/)
- Reverse proxy and handle `/static/` with a webserver and not the included route

### Usage (docker)

- Clone the repo (`--recursive` for submodules)
- `echo -e "DATABASE_URL=postgres://paste:paste@db/paste\n\nROCKET_ADDRESS=0.0.0.0" > .env`
- Copy config.toml to paste.toml and edit `paste.toml`
- `docker-compose up -d`
- `docker exec -ti paste_paste_1 sh /root/post-install.sh`
- `docker exec -ti paste_paste_1 sh /root/start.sh`

To connect to postgres from within docker, run:

```sh
docker exec -ti paste_paste_1 psql
```
