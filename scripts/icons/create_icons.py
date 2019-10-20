#!/usr/bin/env python3

from os import mkdir, rename
from shutil import copy
from sys import argv
from tempfile import TemporaryDirectory
import subprocess
import toml

def main(tmp):
  # print help if wrong num args
  if len(argv) != 3:
    print('usage: create_icons.py [manifest file] [output file]')
    print()
    print('  generates an svg sprite file for Material Design Icons from a manifest')
    return
  # read the manifest
  with open(argv[1]) as f:
    manifest = toml.load(f)
  # create the MDI SVG repo in the temp dir
  repo = create_repo(tmp)
  # make a svg dir
  mkdir(f'{tmp}/svgs')
  # copy all the icons from the repo
  for icon, path in manifest['icons'].items():
    rename(f'{repo}/svg/{path}.svg', f'{tmp}/svgs/{icon}.svg')
  # make a list of all the new paths
  paths = [f'{tmp}/svgs/{icon}.svg' for icon in manifest['icons'].keys()]
  # run svg-sprite
  subprocess.run(['svg-sprite', '-s', '--symbol-dest', f'{tmp}/symbol'] + paths)
  # copy the result
  copy(f'{tmp}/symbol/svg/sprite.symbol.svg', argv[2])

def create_repo(tmp):
  repo = f'{tmp}/repo'
  subprocess.run(['git', 'clone', '--depth=1', 'https://github.com/Templarian/MaterialDesign-SVG', repo])
  return repo

if __name__ == '__main__':
  with TemporaryDirectory() as tmp:
    main(tmp)
