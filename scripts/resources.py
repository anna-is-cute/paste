#!/usr/bin/env python3

from base64 import b64encode
from hashlib import sha256, sha384
from itertools import chain
from pathlib import Path
from shutil import which
from subprocess import run

def integrity_all(paths):
  return { path: integrity(path) for path in paths }

def integrity(path):
  with open(path, 'rb') as f:
    return b64encode(sha384(f.read()).digest()).decode('utf-8')

def hash_str(s):
  return sha256(s.encode('utf-8')).digest()

sass = which('sass')
babel = which('babel')

if sass is None:
  print('warn: could not find sass in path. css will not be compiled')

if babel is None:
  print('warn: could not find babel in path. js will not be compiled')

# web resources path
web_path = Path('webserver/web')
# all static js and css
files = list(chain(
  web_path.glob('static/**/*.js'),
  web_path.glob('static/**/*.css'),
))
src = {
  'css': [
    web_path / 'src/css/style.scss',
    web_path / 'src/css/dark-style.scss',
  ],
  'js': [
    web_path / 'src/js/',
  ],
}
# template files
templates = list(web_path.glob('templates/**/*.html.tera'))

# calculate old integrity hashes to potentially replace
old_integrity = integrity_all(files)

# compile css resources
if sass is not None:
  for css in src['css']:
    print(f'compiling {css}')
    static_css = str(css).replace('scss', 'css').replace('/src/', '/static/')
    run([sass, '-s', 'compressed', f'{css}:{static_css}'])

# compile js resources (assuming directories)
if babel is not None:
  for js in src['js']:
    print(f'compiling {js}')
    static_js = str(js).replace('/src/', '/static/')
    run([babel, '-s', 'true', '-d', static_js, str(js)])

# calculate new integrity hashes
new_integrity = integrity_all(files)

# determine which files changed
changed = {}

for name, ihash in new_integrity.items():
  if name not in old_integrity or ihash == old_integrity[name]:
    continue
  changed[name] = (old_integrity[name], ihash)

# go through every template and replace change hashes if present
for template in templates:
  # read in template
  with open(template, 'r') as f:
    content = f.read()
  # calculate sha256 hash before replacing
  before = hash_str(content)
  # replace any changed hashes
  for name, paths in changed.items():
    (old, new) = paths
    content = content.replace(old, new)
  # calculate sha256 hash after replacing
  after = hash_str(content)
  # if no change, ignore
  if before == after:
    continue
  # update the template with the replaced hashes
  print(f'updating {template}')
  with open(template, 'w') as f:
    f.write(content)
