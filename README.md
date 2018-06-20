# paste

*A sensible, modern pastebin.*

[![build status](https://travis-ci.org/jkcclemens/paste.svg?branch=master)](https://travis-ci.org/jkcclemens/paste)
[![dependency status](https://deps.rs/repo/github/jkcclemens/paste/status.svg)](https://deps.rs/repo/github/jkcclemens/paste)
[![patreon](https://img.shields.io/badge/donate-patreon-blue.svg)](https://www.patreon.com/jkcclemens/overview)
[![paypal](https://img.shields.io/badge/donate-paypal-blue.svg)](https://paypal.me/jkcclemens)

## Idea

Pretty much every pastebin sucks. When Gist removed anonymous pastes, I realised that the pastebins
out there don't do what I want them to do. I made paste to remedy that problem.

There should be a pastebin that's easy-to-use and simple, supporting multiple files, syntax
highlighting, anonymity, and secure authentication. Now there is.

## Status

paste works in its current state. There may be heinous bugs, but I use it as my daily pastebin. It
is not currently *stable*, meaning the API could change in a breaking way at any moment and that the
web interface and routes may change at any moment, as well.

However, these changes are not without good reason (and are usually debated in depth) and are few
and far between.

## Using a pre-existing paste server

I host [paste.gg](https://paste.gg). I am poor and can't afford a good machine (paste.gg is an AWS
t2.micro – one core and low RAM), so please don't bombard it!

## Setting up your own paste server

### You will need

- [Rust](https://rustup.rs/)
- Ruby
- modern postgres (9.x+ tested)
- redis
- sidekiq
- nginx

### Steps

1. Clone the repo (`--recursive` for submodules)
2. `cargo install diesel_cli --no-default-features --features postgres`
3. Copy the example config files
    - `cp Rocket{.example,}.toml; cp config{.example,}.toml; cp sidekiq{.example,}.yml`
4. Edit the config files
5. `cargo build --release`
6. Create a postgres database and user
7. `echo 'DATABASE_URL=postgres://username@/database' > .env`
8. `diesel migration run`
9. Make sure a redis server is running and set the URL in `.env`
10. Start sidekiq using `sidekiq.sh` edited to be correct
11. Preferably use `ROCKET_ENV=prod` and set a secret key in `Rocket.toml`
    - See [Rocket docs](https://rocket.rs/guide/configuration/)
12. `target/release/paste config.toml`
13. Reverse proxy and handle `/static/` with a webserver and not the included route. nginx configuration below.

    ```nginx
    location /static/ {
      alias /path/to/repo/webserver/web/static/;
    }
    ```

## Contact

Join the [Discord server](https://discord.gg/EnqSwJK)!
