#!/usr/bin/env python3
"""Run a pinned independent OIDF suite against an explicitly configured pilot."""
import argparse, json, os, pathlib, subprocess, sys
PIN='440eec8bac7b12b7389d7ca9cbc459b53507a443'
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--suite',type=pathlib.Path,required=True)
parser.add_argument('--config',type=pathlib.Path,required=True)
parser.add_argument('--plan',required=True,help='Exact upstream plan name and variants')
parser.add_argument('--output',type=pathlib.Path,default=pathlib.Path('target/conformance'))
args=parser.parse_args()
for key in ('CONFORMANCE_SERVER','CONFORMANCE_TOKEN'):
    if not os.environ.get(key): raise SystemExit(key+' must be configured for the pilot')
suite=args.suite.resolve(); config=args.config.resolve(); output=args.output.resolve()
revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=suite,text=True).strip()
if revision != PIN: raise SystemExit('Expected independent suite commit '+PIN)
if not config.is_file(): raise SystemExit('Private suite configuration is missing')
output.mkdir(parents=True,exist_ok=True,mode=0o700)
metadata={'schema':'riauth.conformance/v1','suite_commit':revision,'riauth_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'plan':args.plan}
result=subprocess.run([sys.executable,str(suite/'scripts/run-test-plan.py'),'--no-parallel','--export-dir',str(output),args.plan,str(config)],cwd=suite,check=False)
metadata.update(exit_code=result.returncode,passed=result.returncode==0)
(output/'run.json').write_text(json.dumps(metadata,indent=2)+'\n')
raise SystemExit(result.returncode)
