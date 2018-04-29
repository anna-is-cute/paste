#!/usr/bin/env sh

# Check the last rustc version, if any
LAST_RUSTC_VERSION=$(cat "$TRAVIS_BUILD_DIR/target/rustc_version" 2>/dev/null)

# Get the new rustc version
CURR_RUSTC_VERSION=$(rustc --version)

# If the last rustc version doesn't match the current, rustc has been upgraded. This will cause all
# dependencies to be rebuilt, bloating the cache. Clean out all the old dependency files in this
# case.
if [ "$LAST_RUSTC_VERSION" != "$CURR_RUSTC_VERSION" ]; then
  echo "rustc version discrepancy (\"$LAST_RUSTC_VERSION\" != \"$CURR_RUSTC_VERSION\"). running cargo clean"
  cargo clean
fi

# Recreate the target directory, if necessary.
mkdir -p "$TRAVIS_BUILD_DIR/target" 2>/dev/null

# Update the last rustc version to the current rustc version.
echo "$CURR_RUSTC_VERSION" > "$TRAVIS_BUILD_DIR/target/rustc_version"
