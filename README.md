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

## Setting up your own paste server (Docker)

You can run your own paste server by using `docker-compose`. You will need Docker, obviously.

The Docker image will build and run (in release mode) the server as it is in the repository. That is
to say that whatever you have checked out will be built and run.

### Edit configuration files

First, you'll need to set up your configuration. All configuration files are missing by default. You
will need to copy over their examples files and edit those. The example files follow this pattern:

If you need `./Rocket.toml`, the example file will be located at `./Rocket.example.toml`.

1. `.docker/nginx/nginx.conf`

    Change anything you deem necessary.

2. `.docker/nginx/sites`

    Choose the file that is best for you. There are HTTP and HTTPS configurations. nginx will load
    anything that matches `*.conf` in that directory, so rename or copy whichever you want.

    File is commented with what changes are necessary and notes.

3. `Rocket.toml`

    Read the comments. Set a secret key. The repo is included in this Docker container, so certs can
    be specified at `.docker/nginx/run/certs`.

4. `config.toml`

    Change everything, basically. Read the comments.

5. `sidekiq.yml`

    Probably fine, but change queue concurrency if you need to.

6. `sidekiq.sh`

    This is a convenience script for you. Change the variables at the start for starting sidekiq
    properly to work with paste.

7. `.env`

    Change this to the below.

    ```shell
    DATABASE_URL=postgres://paste:paste@postgres/paste
    REDIS_URL=redis://redis
    SIDEKIQ_URL=redis://redis/1
    EMAIL_TEMPLATES=webserver/web/emails/*
    ```

### Start the server

```sh
# from the repo root
docker-compose -f .docker/docker-compose.yml up
```

If everything is configured correctly, it should start up, and you should see `Rocket has launched
from https://0.0.0.0:8000`. Note that this is not how the outside world will access paste.

nginx is set up to expose ports 80 and 443 on the host by default, but you can change
`docker-compose.yml` (and you'll need to change `nginx.conf` as well) to change that.

Access paste by connecting to the host via HTTP or HTTPS, depending on how you set up nginx.

Done!

## Setting up your own paste server (manual)

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
8. `diesel migration run --migration-dir=webserver/migrations`
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
