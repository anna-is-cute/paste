# paste

*A sensible, modern pastebin.*

[![build status](https://travis-ci.org/jkcclemens/paste.svg?branch=master "travis-ci status")](https://travis-ci.org/jkcclemens/paste)
[![pipeline status](https://gitlab.com/jkcclemens/paste/badges/master/pipeline.svg "gitlab ci status")](https://gitlab.com/jkcclemens/paste/commits/master)
[![dependency status](https://deps.rs/repo/github/jkcclemens/paste/status.svg)](https://deps.rs/repo/github/jkcclemens/paste)
[![patreon](https://img.shields.io/badge/donate-patreon-blue.svg)](https://www.patreon.com/jkcclemens/overview)
[![paypal](https://img.shields.io/badge/donate-paypal-blue.svg)](https://paypal.me/jkcclemens)
[![discord](https://img.shields.io/badge/contact-discord-7289DA.svg)](https://discord.gg/EnqSwJK)

<img src="docs/light_screenshot.png" width="430" alt="Light theme screenshot"> <img src="docs/dark_screenshot.png" width="430" alt="Dark theme screenshot">

## Idea

Pretty much every pastebin sucks. When Gist removed anonymous pastes, I realised that the pastebins
out there don't do what I want them to do. I made paste to remedy that problem.

There should be a pastebin that's easy-to-use and simple, supporting multiple files, syntax
highlighting, anonymity, and secure authentication. Now there is.

## Status

paste's API is stable. Any breaking changes, including route renames, will come with a version bump
to the API.

paste's web interface, backend, routes, and code are all unstable. There may be heinous bugs.
However, I would note that both myself and a few others use paste as our daily pastebins without
issue.

## Contact

Join the [Discord server](https://discord.gg/EnqSwJK) for questions, comments, and general paste-y
community!

## Contributing

**Important**: Ensure you clone the repository with submodules and [git-lfs](https://git-lfs.github.com/)!  
Make sure git-lfs is installed, then run `git clone --recurse-submodules https://github.com/ascclemens/paste.git`.

To set up a development environment with a debug build of paste, use `paste.sh` in the script
directory! Just run `scripts/paste.sh` by itself to show help.

- Run `scripts/paste.sh start` to start the server and generate any missing config files and a
  self-signed certificate.
- If this is your first run, paste will need to build `diesel_cli` and `paste` itself, which can take some time.
  - Use `scripts/paste.sh logs` to keep track of the compilation. Note that sidekiq will fail to start until compilation completes. You will be able to access nginx, but you will receive a gateway error until paste finishes compiling and starts.
- Once Rocket says it's ready, access `https://localhost` and accept the self-signed certificate.

Changes to the HTML templates and static resources should take effect on reload, but changes to any Rust will need a
`scripts/paste.sh compose restart backend` to recompile.

## Using a pre-existing paste server

I host [paste.gg](https://paste.gg). I am poor and can't afford a good machine (paste.gg is a DigitalOcean droplet – three cores and low-ish RAM), so please don't bombard it!

## Setting up your own paste server (production, Docker)

You can run your own paste server by using `docker-compose`. You will need Docker, obviously.

The Docker image will build and run (in release mode) the server as it is in the repository. That is
to say that whatever you have checked out will be built and run.

### Edit configuration files

First, you'll need to set up your configuration. All configuration files are missing by default. You
will need to copy over their examples files and edit those. The example files follow this pattern:

If you need `./Rocket.toml`, the example file will be located at `./Rocket.example.toml`.

1. `.docker/nginx/nginx.conf`

    Change anything you deem necessary.

2. `.docker/nginx/sites/paste.conf`

    This one breaks the naming scheme, since nginx will load anything that matches `*.conf` in that
    directory!

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
    # if you want to enable camo, fill out the two variables below
    # the camo url will be accessible at your externally-facing hostname, so change the name below
    # CAMO_URL=https://your.hostname/camo/
    # this should be a random string
    # CAMO_KEY=BNdLplouhYWXEFHlK2v7pO8oxOcbCL3iShURSD5HVhpaJOsyYFsK7yjZx6CSYPSA
    ```

### Start the server

```sh
# from the repo root
docker-compose -p paste -f .docker/docker-compose.yml up
```

If everything is configured correctly, it should start up, and you should see `Rocket has launched
from https://0.0.0.0:8000`. Note that this is not how the outside world will access paste.

nginx is set up to expose ports 80 and 443 on the host by default, but you can change
`docker-compose.yml` (and you'll need to change `nginx.conf` as well) to change that.

Access paste by connecting to the host via HTTPS.

Done!

## Setting up your own paste server (production, manual)

### You will need

- [Rust](https://rustup.rs/) (nightly channel required)
- Ruby
- modern postgres (9.x+ tested)
- redis
- sidekiq
- nginx
- camo (optional)

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
11. (Optional) Start a camo server and edit .env to contain the secret key and the public-facing URL
    for it.
12. Preferably use `ROCKET_ENV=prod` and set a secret key in `Rocket.toml`
    - See [Rocket docs](https://rocket.rs/guide/configuration/)
13. `target/release/paste config.toml`
14. Reverse proxy and handle `/static/` with a webserver and not the included route. nginx configuration below.

    ```nginx
    location /static/ {
      alias /path/to/repo/webserver/web/static/;
    }
    ```
    
## Command-line client

A compatible and recommended command-line client for paste is [bins](https://github.com/jkcclemens/bins).

## Compiling

If you just want to compile paste, perhaps to test while making a contribution, there is a Docker
image that has a set-up build environment.

```shell
# from repo root
docker run --rm -it -v "$(pwd)":/paste jkcclemens/paste
```

By default, that will build paste, but you can also append `/bin/bash` to the command to start a
Bash shell in the working environment.

## API

See the [paste API documentation](/api.md).
