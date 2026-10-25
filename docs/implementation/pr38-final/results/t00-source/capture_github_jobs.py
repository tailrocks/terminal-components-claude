#!/usr/bin/env python3
"""Capture paginated public GitHub Actions job responses without credentials."""
import datetime
import hashlib
import json
import pathlib
import collections
import sys
import urllib.error
import urllib.request

REPO='tailrocks/terminal-components-claude'
RUN_ID=40235807469
SOURCE='1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
OUT=pathlib.Path('/Users/donbeave/Downloads/terminal-components-claude/docs/implementation/pr38-final/results/t00-source')

pages=[]; total=None; page=1; failed=False
run_url=f'https://api.github.com/repos/{REPO}/actions/runs/{RUN_ID}'
run_req=urllib.request.Request(run_url,headers={'Accept':'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28','User-Agent':'T00-source-capture'})
run_observed=datetime.datetime.now(datetime.timezone.utc).isoformat()
try:
 with urllib.request.urlopen(run_req,timeout=25) as response:
  run_status=response.status; run_headers=dict(response.headers.items()); run_body=response.read()
 run_payload=json.loads(run_body)
except urllib.error.HTTPError as error:
 run_status=error.code; run_headers=dict(error.headers.items()); run_body=error.read(); run_payload=None; failed=True
except Exception as error:
 run_status=None; run_headers={}; run_body=b''; run_payload=None; failed=True
 run_error=f'{type(error).__name__}: {error}'
while True:
 url=f'https://api.github.com/repos/{REPO}/actions/runs/{RUN_ID}/jobs?per_page=100&page={page}'
 req=urllib.request.Request(url,headers={'Accept':'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28','User-Agent':'T00-source-capture'})
 observed=datetime.datetime.now(datetime.timezone.utc).isoformat()
 try:
  with urllib.request.urlopen(req,timeout=25) as response:
   status=response.status; headers=dict(response.headers.items()); body=response.read()
 except urllib.error.HTTPError as error:
  status=error.code; headers=dict(error.headers.items()); body=error.read(); failed=True
 except Exception as error:
  pages.append({'page':page,'request_url':url,'observed_at_utc':observed,'transport_error':f'{type(error).__name__}: {error}'})
  failed=True; break
 digest=hashlib.sha256(body).hexdigest()
 try: payload=json.loads(body)
 except Exception: payload=None; failed=True
 jobs=payload.get('jobs',[]) if isinstance(payload,dict) else []
 if isinstance(payload,dict) and isinstance(payload.get('total_count'),int): total=payload['total_count']
 pages.append({'page':page,'request_url':url,'observed_at_utc':observed,'http_status':status,'response_headers':{k:v for k,v in headers.items() if k.lower() in {'date','link','x-ratelimit-limit','x-ratelimit-remaining','x-ratelimit-reset','x-github-request-id'}},'response_sha256':digest,'response_bytes':len(body),'response':payload})
 if status<200 or status>=300: failed=True; break
 if not jobs: break
 if total is not None and sum(len(x.get('response',{}).get('jobs',[])) for x in pages if isinstance(x.get('response'),dict))>=total: break
 if len(jobs)<100: break
 page+=1

all_jobs=[job for x in pages if isinstance(x.get('response'),dict) for job in x['response'].get('jobs',[])]
step_counts=collections.Counter(); failed_steps=[]; test_step_results=[]; test_build_results=[]; production_build_steps=[]
for job in all_jobs:
 for step in job.get('steps') or []:
  key=f"{step.get('name','unknown')}/{step.get('status','unknown')}/{step.get('conclusion') or 'pending'}"
  step_counts[key]+=1
  if step.get('conclusion')=='failure': failed_steps.append({'job_id':job.get('id'),'job_name':job.get('name'),'step':step.get('name'),'status':step.get('status'),'conclusion':step.get('conclusion')})
  if step.get('name')=='Unit and integration tests': test_step_results.append({'job_id':job.get('id'),'job_name':job.get('name'),'job_conclusion':job.get('conclusion'),'step_status':step.get('status'),'step_conclusion':step.get('conclusion')})
  if step.get('name')=='Build test executables': test_build_results.append({'job_id':job.get('id'),'job_name':job.get('name'),'job_conclusion':job.get('conclusion'),'step_status':step.get('status'),'step_conclusion':step.get('conclusion')})
  if 'production build' in step.get('name','').lower(): production_build_steps.append({'job_id':job.get('id'),'job_name':job.get('name'),'step':step.get('name'),'status':step.get('status'),'conclusion':step.get('conclusion')})
record={'repository':REPO,'source_commit':SOURCE,'run_id':RUN_ID,'run_detail_request_url':run_url,'run_detail_observed_at_utc':run_observed,'run_detail_http_status':run_status,'run_detail_response_sha256':hashlib.sha256(run_body).hexdigest(),'run_detail_response_bytes':len(run_body),'run_detail_response_headers':{k:v for k,v in run_headers.items() if k.lower() in {'date','x-ratelimit-limit','x-ratelimit-remaining','x-ratelimit-reset','x-github-request-id'}},'run_detail':run_payload,'run_detail_error':locals().get('run_error'),'pages':pages,'total_count':total,'jobs_returned':len(all_jobs),'pagination_complete':bool(not failed and total is not None and len(all_jobs)==total),'job_status_counts':{},'step_status_counts':dict(step_counts),'failed_steps':failed_steps,'unit_and_integration_step_results':test_step_results,'test_executable_build_step_results':test_build_results,'explicit_production_build_step_results':production_build_steps,'production_build_assessment':'No step named as a production build was reported. Build test executables is a separate test-artifact build. Cancelled jobs leave production-build outcomes unknown.','limits':['This is one timestamped remote snapshot. In-progress job states can change after capture.','A complete job page does not imply completed builds or tests.','A successful test step inside a cancelled job is a step-level result; the whole job is still cancelled.']}
for job in all_jobs:
 key=f"{job.get('status','unknown')}/{job.get('conclusion') or 'pending'}"
 record['job_status_counts'][key]=record['job_status_counts'].get(key,0)+1
path=OUT/'ci-head-1d6-rest.json'
path.write_text(json.dumps(record,ensure_ascii=False,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(json.dumps({'path':str(path),'run_id':RUN_ID,'total_count':total,'jobs_returned':len(all_jobs),'pagination_complete':record['pagination_complete'],'job_status_counts':record['job_status_counts'],'http_statuses':[x.get('http_status') for x in pages]},sort_keys=True))
sys.exit(1 if failed else 0)
