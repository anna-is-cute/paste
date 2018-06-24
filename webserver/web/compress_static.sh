#!/bin/sh

FILES=$(find static/ -type f)

for f in $FILES; do
  if [ -f "$f.br" ]; then
    echo "Deleting $f.br"
    rm -f "$f.br"
  fi
  echo "Compressing $f..."
  brotli --quality 11 --input "$f" --output "$f.br"
  chmod 0644 "$f.br"
done
