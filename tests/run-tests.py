#!/usr/bin/env python
import glob
import re
import os
import argparse
import time
import asyncio

THIS_FILE = os.path.dirname(os.path.realpath(__file__))
TEMP_DIR = os.path.join(THIS_FILE, '.temp')
RUN_REGEX = re.compile(r'RUN:\s*(.+)')

SUBS = {
  '%s': lambda source: source,
  '%t': lambda _: TEMP_DIR,
  'run-kate': lambda _: 'cargo run --'
}

def make_parser():
  parser = argparse.ArgumentParser(
    prog='run-tests',
    description='Runs the test suite',
  )

  parser.add_argument('--verbose', '-v', action='store_true')
  parser.add_argument('--filter', '-f', default=".*")

  return parser

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

def filter_sources(sources, excluded_filename):
  reg = re.compile(excluded_filename)
  def filter_one(source):
    return reg.match(source) != None

  return list(filter(filter_one, sources))

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

async def run_cmd(cmd, display_command):
  if display_command:
    print(f'{Colours.BLUE}run:{Colours.END}', cmd)

  return await asyncio.create_subprocess_shell(
    cmd,
    stdout=asyncio.subprocess.PIPE,
    stderr=asyncio.subprocess.PIPE
  )

async def display_failure(exec, context):
  print(f"""{Colours.RED}{Colours.UNDERLINE}fail: {context}{Colours.END}
{Colours.RED}stdout:{Colours.END}
{(await exec.stdout.read()).decode('utf-8')}
{Colours.RED}stderr:{Colours.END}
{(await exec.stderr.read()).decode('utf-8')}
  """)

async def run_tests(sources):
  passes = []
  fails = []

  if not len(sources):
    print(f'{Colours.WARNING}warn:{Colours.END} no sources found')
    return passes, fails

  async def run_for_source(source_location):
    with open(source_location) as source:
      content = source.read()
      run_cmds = map(
        lambda run_cmd: perform_subs(run_cmd, SUBS, source_location), 
        extract_runners(content)
      )

      procs = []
      exits = []
      start = time.time()

      async for proc in (await run_cmd(run, False) for run in run_cmds):
        exit = await proc.wait()
        procs.append(proc)
        exits.append(exit)

      end = time.time()

      return source_location, (end - start), list(zip(procs, exits))
  
  results = await asyncio.gather(*[
    run_for_source(source) for source in sources
  ])

  for source, duration, execution_set in results:
    did_any_fail = False
    for proc, exit in execution_set:
      if exit != 0:
        did_any_fail = True
        print(await display_failure(proc, source))

    if did_any_fail:
        fails.append(source)
    else:
        print(f'{Colours.GREEN}ok ({round(duration, 3)}s):{Colours.END}', source)
        passes.append(source)

  return passes, fails


async def main():
  args = make_parser().parse_args()
  print(f'{Colours.GREEN}building...{Colours.END}')
  build = await run_cmd("cargo build", args.verbose)
  build = await build.wait()
  if build != 0:
    await display_failure(build, "build")
    return

  print(f'{Colours.GREEN}cleaning...{Colours.END}')
  clean = await run_cmd(f'rm -rf {TEMP_DIR}', args.verbose)
  clean = await clean.wait()
  if clean != 0:
    await display_failure(clean, "clean")
    return
  
  print(f'{Colours.GREEN}pre-run steps ok{Colours.END}')
  print()
  print(f'{Colours.GREEN}running...{Colours.END}')
  print()

  start = time.time()
  passes, fails = await run_tests(filter_sources(get_sources(['java']), args.filter))
  end = time.time()
  duration = end - start

  print()
  print()
  print(f'{Colours.PEACH}testing concluded{Colours.END} ({round(duration, 3)}s)')
  print(f'{Colours.GREEN}pass:{Colours.END} {len(passes)} - {Colours.RED}fail:{Colours.END} {len(fails)}')

if __name__ == '__main__':
  asyncio.run(main())