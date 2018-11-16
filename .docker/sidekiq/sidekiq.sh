#!/bin/bash

# Edit these variables.

# The environment to start sidekiq in. "development" "beta" "production"
SIDEKIQ_ENV="production"
# The sidekiq config file path
CONF="sidekiq.yml"
# The .env file path with SIDEKIQ_URL defined
ENV_FILE=".env"

# Don't edit below here.

LIBRARIES=/libraries/
SHASUMS=$LIBRARIES/shasums

# Wait for the shasums file
while [ ! -f $SHASUMS ]; do sleep 1; done

# check the checksums
while IFS=' ' read -r checksum name; do
  # strip off ./
  name=$(basename "$name")
  local_path=$LIBRARIES/$name
  # wait for the file to exist
  while [ ! -f "$local_path" ]; do sleep 1; done
  # try to validate checksum three times
  for i in {1..3}; do
    local_checksum=$(sha256sum "$local_path" | cut -d' ' -f1)
    [ "$local_checksum" == "$checksum" ] && break
    if [ "$i" == "3" ]; then
      echo "could not verify integrity of $name. expected checksum $checksum but got $local_checksum."
      exit 1
    fi
    sleep 1
  done
done < $SHASUMS

PASTE=/paste/

source "$PASTE/$ENV_FILE"
unset REDIS_URL

LD_LIBRARY_PATH="$LIBRARIES" \
REDIS_URL="$SIDEKIQ_URL" \
exec sidekiq \
  -C "$PASTE/$CONF" \
  -r "$PASTE/webserver/workers.rb" \
  -e "$SIDEKIQ_ENV"
