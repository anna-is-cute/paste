#!/bin/sh

# DOCKER USERS: please change #!/bin/sh to #!/bin/bash :)

# Edit these variables.

# The environment to start sidekiq in. "development" "beta" "production"
SIDEKIQ_ENV="development"
# Either "debug" or "release" – whichever you chose when compiling the workers
RUST_ENV="release"
# The sidekiq config file path
CONF="sidekiq.yml"
# The .env file path with SIDEKIQ_URL defined
ENV_FILE=".env"
# The path to the target directory (this should be ./.docker/run/target for docker)
TARGET_DIR="./target"

# Don't edit below here.

source "$ENV_FILE"
unset REDIS_URL

LD_LIBRARY_PATH="./$TARGET_DIR/$RUST_ENV" \
REDIS_URL="$SIDEKIQ_URL" \
sidekiq \
  -C "$CONF" \
  -r ./webserver/workers.rb \
  -e "$SIDEKIQ_ENV"
