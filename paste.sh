#!/bin/sh

###
### Logging
###

stderr() {
  echo "$@" >&2
}

# emit a log message to stderr
# args(1): log level
# args(..): message
log() {
  level="$1"
  shift
  stderr "$level: $*"
}

# emit an info message to stderr
# args(..): message
info() {
  log info "$@"
}

# emit a warning message to stderr
# args(..): message
warn() {
  log warn "$@"
}

# emit an error message to stderr
# args(..): message
error() {
  log error "$@"
}

# emit an error message and exit unsuccessfully
# args(..): message
die() {
  error "$@"
  exit 1
}

###
### Dependencies
###

# ensure that the dependencies for using this script are installed or exit
ensure_deps() {
  info "checking for dependencies"

  # need docker for running the containers
  [ ! -x "$(command -v docker)" ] && die "missing docker"
  # need docker-compose for orchestrating containers
  [ ! -x "$(command -v docker-compose)" ] && die "missing docker-compose"
  # need openssl for generating cert
  [ ! -x "$(command -v openssl)" ] && die "missing openssl"
  # need sed for doing substitutions
  [ ! -x "$(command -v sed)" ] && die "missing sed"
}

###
### Certificates
###

CERT_LOC=".docker/nginx/certs"
CERT_KEY="$CERT_LOC/privkey.pem"
CERT_CERT="$CERT_LOC/fullchain.pem"

# generate a self-signed certificate
# args(1): common name
# args(2): private key location
# args(3): certificate location
generate_cert() {
  info "generating a self-signed certificate for $1"

  openssl req \
    -x509 \
    -new \
    -newkey rsa:4096 \
    -keyout "$2" \
    -out "$3" \
    -days 365 \
    -nodes \
    -subj "/CN=$1"
}

does_cert_exist() {
  [ -f $CERT_CERT ] && [ -f $CERT_KEY ]
}

make_cert_if_missing() {
  does_cert_exist && return

  generate_cert "localhost" "$CERT_KEY" "$CERT_CERT"
}

###
### Configs
###

configs() {
  info "setting up any missing configuration files"
  config_nginx
  config_nginx_site
  config_rocket
  config_paste
  config_sidekiq_yml
  config_sidekiq_sh
  config_env
}

config_nginx() {
  nginx_loc=".docker/nginx/nginx.conf"
  # don't overwite
  [ -f "$nginx_loc" ] && return
  info "setting up nginx.conf"
  # copy the example config
  cp ".docker/nginx/nginx.example.conf" "$nginx_loc"
  # default config is fine
}

config_nginx_site() {
  # define where the result config will be
  nginx_loc=".docker/nginx/sites/localhost.conf"
  # don't overwrite existing
  [ -f "$nginx_loc" ] && return
  info "setting up nginx - localhost.conf"
  # copy the example config
  cp ".docker/nginx/sites/paste.conf.example" "$nginx_loc"
  # change the server name anywhere it's used
  sed -i 's/change\.me/localhost/g' "$nginx_loc"
  # don't need to change cert locs because they're already in the right place
}

config_rocket() {
  # don't overwrite existing
  [ -f "Rocket.toml" ] && return
  info "setting up Rocket.toml"
  # copy the example config
  cp "Rocket.example.toml" "Rocket.toml"
  # uncomment the address and port lines
  sed -i 's/# address =/address =/g' "Rocket.toml"
  sed -i 's/# port =/port =/g' "Rocket.toml"
  # set the key paths
  sed -i 's/path\/to\/certs.pem/.docker\/nginx\/certs\/fullchain.pem/g' "Rocket.toml"
  sed -i 's/path\/to\/key.pem/.docker\/nginx\/certs\/privkey.pem/g' "Rocket.toml"
  # set the templates path
  sed -i 's/web\/templates/webserver\/web\/templates/g' "Rocket.toml"
}

config_paste() {
  # don't overwrite existing config
  [ -f "config.toml" ] && return
  info "setting up config.toml"
  # copy the example config
  cp "config.example.toml" "config.toml"
  # set the site name
  sed -i 's/"paste"/"paste dev"/g' "config.toml"
  # set the site address
  sed -i 's/paste\.gg/localhost/g' "config.toml"
  # set the store path to "/store" for docker
  sed -i 's/\.\/store/\/store/g' "config.toml"
  # don't set up email
}

config_sidekiq_yml() {
  # don't overwrite existing config
  [ -f "sidekiq.yml" ] && return
  info "setting up sidekiq.yml"
  # copy the example config
  cp "sidekiq.example.yml" "sidekiq.yml"
  # defaults are fine
}

config_sidekiq_sh() {
  # don't overwrite existing config
  [ -f "sidekiq.sh" ] && return
  info "setting up sidekiq.sh"
  # copy the example config
  cp "sidekiq.example.sh" "sidekiq.sh"
  # we're using debug
  sed -i 's/RUST_ENV="release"/RUST_ENV="debug"/g' "sidekiq.sh"
  # set the target dir
  sed -i 's/TARGET_DIR="\.\/target"/TARGET_DIR=".\/.docker\/run\/target"/g' "sidekiq.sh"
}

config_env() {
  # don't overwrite existing config
  [ -f ".env" ] && return
  info "setting up .env"
  rand=$(openssl rand -hex 32)
  # set up the docker .env
  cat > .env <<D_ENV
DATABASE_URL=postgres://paste:paste@postgres/paste
REDIS_URL=redis://redis
SIDEKIQ_URL=redis://redis/1
EMAIL_TEMPLATES=webserver/web/emails/*
# if you want to enable camo, fill out the two variables below
# the camo url will be accessible at your externally-facing hostname, so change the name below
CAMO_URL=https://localhost/camo/
# this should be a random string
CAMO_KEY=camo_key
D_ENV
  # replace camo key
  sed -i "s/camo_key/$rand/g" ".env"
}

###
### Docker
###

docker_start() {
  info "starting paste"
  compose up --build -d
}

docker_stop() {
  info "stopping paste"
  compose stop
}

###
### Subcommands
###

prepare() {
  # first make sure we have the necessary dependencies installed
  ensure_deps
  # generate a cert if there is none
  make_cert_if_missing
  # create the configs
  configs
}

start() {
  # set up necessary files
  prepare
  # start docker up
  docker_start
}

stop() {
  docker_stop
}

restart() {
  stop
  start
}

logs() {
  compose logs -f --tail=5
}

compose() {
  if groups | grep docker; then
    sudo=""
  else
    # shellcheck disable=SC2016
    warn 'you may be asked for your password for `sudo docker-compose`'
    sudo="sudo"
  fi
  "$sudo" docker-compose -p paste-dev -f .docker/docker-compose.development.yml "$@"
}

show_help() {
  stderr "usage: ./paste.sh [start|stop|restart|logs|compose|help] (args)"
  stderr
  stderr "  start"
  stderr "    create dev config files if necessary and start paste"
  stderr "  stop"
  stderr "    stop paste"
  stderr "  restart"
  stderr "    stop then start paste"
  stderr "  logs"
  stderr "    view logs for all services"
  stderr "  compose (args)"
  stderr "    run docker-compose with the given args (already using config file and name)"
  stderr "  help"
  stderr "    display this help"
}

###
### Main
###

main() {
  if [ "$#" -eq 0 ]; then
    show_help
    exit 1
  fi

  sub="$1"
  shift
  case "$sub" in
    "start")
      start
      ;;
    "stop")
      stop
      ;;
    "restart")
      restart
      ;;
    "logs")
      logs
      ;;
    "compose")
      compose "$@"
      ;;
    "help")
      show_help
      ;;
  esac
}

main "$@"
