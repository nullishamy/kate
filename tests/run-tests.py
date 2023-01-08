#!/usr/bin/env python
import glob
import re
import subprocess
import os

THIS_FILE = os.path.dirname(os.path.realpath(__file__))
RUN_REGEX = re.compile(r'RUN:\s*(.+)')
SUBS = {
  '%s': lambda source: source,
  '%t': lambda _: f'./.temp',
  'kate': lambda _: '../target/debug/cli'
}

class Colours:
    PEACH = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    WARNING = '\033[93m'
    RED = '\033[91m'
    END = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def get_sources(types):
  files_grabbed = []
  for type in types:
      pat = f'**/*.{type}'
      files_grabbed.extend(glob.glob(pat, recursive=True))

  return files_grabbed

def extract_runners(content):
  return re.findall(RUN_REGEX, content)

def perform_subs(run_cmd, subs, *args):
  tmp = run_cmd
  for (key, fn) in subs.items():
    tmp = tmp.replace(key, fn(*args))

  return tmp

def run_cmd(cmd):
  return subprocess.run(cmd, capture_output=True, shell=True)

def display_failure(exec):
  print(f"""{Colours.RED}{Colours.UNDERLINE}fail:{Colours.END}
{Colours.RED}stdout:{Colours.END}
{exec.stdout.decode('utf-8')}
{Colours.RED}stderr:{Colours.END}
{exec.stderr.decode('utf-8')}
  """)

def run_tests(sources):
  passes = []
  fails = []

  if not len(sources):
    print(f'{Colours.WARNING}warn:{Colours.END} no sources found')
    return

  for source_location in sources:
    print(f'{Colours.PEACH}test:{Colours.END}', source_location)
    with open(source_location) as source:
      content = source.read()
      run_cmds = map(
        lambda run_cmd: perform_subs(run_cmd, SUBS, source_location), 
        extract_runners(content)
      )

      for run in run_cmds:
        print(f'{Colours.BLUE}run:{Colours.END}', run)
        did_fail = False

        exec = run_cmd(run)
        if exec.returncode != 0:
          display_failure(exec)
          did_fail = True
      
      if did_fail:
          fails.append(source_location)
      else:
          print(f'{Colours.GREEN}ok:{Colours.END}', source_location)
          passes.append(source_location)

  return passes, fails


def main():
  print(f'{Colours.GREEN}building...{Colours.END}')
  build = run_cmd("cargo build")
  if build.returncode != 0:
    display_failure(build)
    return

  print(f'{Colours.GREEN}cleaning...{Colours.END}')
  clean = run_cmd('rm -rf .temp')
  if clean.returncode != 0:
    display_failure(clean)
    return
  
  print(f'{Colours.GREEN}pre-run steps ok{Colours.END}')
  print()

  passes, fails = run_tests(get_sources(['java']))
  print(f'{Colours.PEACH}testing concluded{Colours.END}')
  print(f'{Colours.GREEN}pass:{Colours.END} {len(passes)} - {Colours.RED}fail:{Colours.END} {len(fails)}')

main()