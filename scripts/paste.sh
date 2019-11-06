#!/bin/sh

# This script manages a development instance of paste. It will automatically generate config files
# from the example files, and it will automatically generate TLS certificates for localhost.
#
# It can be used to run any docker-compose command, as well.
#
# This script should be run from the repository root always (`scripts/paste.sh`).
#
# To get started, run `paste.sh start` and `paste.sh logs`.

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
  log "$(styled cyan info)" "$@"
}

# emit a warning message to stderr
# args(..): message
warn() {
  log "$(styled yellow warn)" "$@"
}

# emit an error message to stderr
# args(..): message
error() {
  log "$(styled red error)" "$@"
}

# emit an error message and exit unsuccessfully
# args(..): message
die() {
  error "$@"
  exit 1
}

STYLE_RED="$(tput setaf 1)"
STYLE_GREEN="$(tput setaf 2)"
STYLE_YELLOW="$(tput setaf 3)"
STYLE_CYAN="$(tput setaf 6)"
STYLE_BOLD="$(tput bold)"
STYLE_DIM="$(tput dim)"

# apply various styles to the input
# args(1): space-separated styles
# args(..): text to style
styled() {
  s=""
  desc="$1"
  shift
  for style in $desc; do
    case "$style" in
      "red") s="$s$STYLE_RED" ;;
      "green") s="$s$STYLE_GREEN" ;;
      "yellow") s="$s$STYLE_YELLOW" ;;
      "cyan") s="$s$STYLE_CYAN" ;;
      "bold") s="$s$STYLE_BOLD" ;;
      "dim") s="$s$STYLE_DIM" ;;
    esac
  done
  echo "$s$*$(tput sgr0)"
}

###
### Dependencies
###

# ensure that the dependencies for using this script are installed or exit
ensure_deps() {
  info "checking for dependencies"

  should_die=0

  # need docker for running the containers
  [ ! -x "$(command -v docker)" ] && error "missing docker" && should_die=1
  # need docker-compose for orchestrating containers
  [ ! -x "$(command -v docker-compose)" ] && error "missing docker-compose" && should_die=1
  # need openssl for generating cert
  [ ! -x "$(command -v openssl)" ] && error "missing openssl" && should_die=1
  # need sed for doing substitutions
  [ ! -x "$(command -v sed)" ] && error "missing sed" && should_die=1
  # need groups to check for docker group
  [ ! -x "$(command -v groups)" ] && error "missing groups" && should_die=1
  # need grep to check groups
  [ ! -x "$(command -v grep)" ] && error "missing grep" && should_die=1

  [ "$should_die" -eq 1 ] && exit 1
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
  [ -f "$CERT_CERT" ] && [ -f "$CERT_KEY" ]
}

is_cert_good_for_a_week() {
  # check if the cert is expired or will expire in less than a week
  openssl x509 -checkend 604800 -noout -in "$CERT_CERT" >/dev/null
}

make_cert_if_necessary() {
  info "checking certificate status"

  does_cert_exist && is_cert_good_for_a_week && return

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
  # set up to show errors, since we're doing development
  sed -i 's/error_log \/dev\/null emerg/error_log error.log error/g' "$nginx_loc"
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
  # don't change the store path, as we're not using a volume
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
  # replace the shebang
  sed -i 's/\/bin\/sh/\/bin\/bash/g' "sidekiq.sh"
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
  # generate a cert if there is none
  make_cert_if_necessary
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
  if groups | grep docker >/dev/null; then
    sudo=""
  else
    # shellcheck disable=SC2016
    warn 'you may be asked for your password for `sudo docker-compose`'
    sudo="sudo"
  fi
  $sudo docker-compose -p paste-dev -f .docker/docker-compose.development.yml "$@"
}

show_help() {
  stderr "$(styled red usage): $0 [start|stop|restart|logs|compose|help] (args)"
  stderr
  stderr "this script manages a *$(styled 'yellow bold' development)* instance of paste, setting up"
  stderr "configuration files and certificates automatically. it should be run"
  # shellcheck disable=SC2016
  stderr 'from the repository root (usually `scripts/paste.sh`)'
  stderr
  stderr "  $(styled green start)"
  stderr "    create dev config files if necessary and start paste"
  stderr "  $(styled green stop)"
  stderr "    stop paste"
  stderr "  $(styled green restart)"
  stderr "    stop then start paste"
  stderr "  $(styled green logs)"
  stderr "    view logs for all services"
  stderr "  $(styled green compose) $(styled dim '(args)')"
  stderr "    run docker-compose with the given args (already using config file and name)"
  stderr "    $(styled cyan ex): $0 compose restart backend"
  stderr "  $(styled green help)"
  stderr "    display this help"
}

###
### Main
###

main() {
  # show help on no args
  if [ "$#" -eq 0 ]; then
    show_help
    exit 1
  fi

  # get first arg
  sub="$1"
  shift

  # handle help before anything else, since everything else requires ensuring dependencies
  if [ "$sub" = "help" ]; then
    show_help
    return
  fi

  # ensure dependencies are installed
  ensure_deps

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
    *)
      die "bad subcommand"
      ;;
  esac
}

main "$@"
