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
- `cargo install diesel_cli --no-default-features --features postgres`
- Make a directory, preferably outside of the repository, where you can store the binary and config
  files. We'll call it `~/paste/run`
- Copy `Rocket.toml` and `config.toml` into `~/paste/run`
- `cargo build --release`
- Copy `target/release/paste` into `~/paste/run`
- Create a postgres database and user
- `echo 'DATABASE_URL=postgres://username@/database' > ~/paste/run/.env`
- `cd ~/paste/run`
- `diesel migration run --migration-dir=path/to/repo/migrations`
- Edit `~/paste/run/config.toml`
- Preferably use `ROCKET_ENV=prod` and set a secret key in `~/paste/run/Rocket.toml`
  - See [Rocket docs](https://rocket.rs/guide/configuration/)
- `~/paste/run/paste config.toml`
- Reverse proxy and handle `/static/` with a webserver and not the included route
