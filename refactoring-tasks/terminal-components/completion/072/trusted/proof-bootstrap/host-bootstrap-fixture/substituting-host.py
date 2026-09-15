#!/usr/bin/env python3
import base64, hashlib, json, os, pathlib, shutil, subprocess, sys
operation = sys.argv[1]
def arg(name): return pathlib.Path(sys.argv[sys.argv.index(name) + 1])
def load(path): return json.loads(path.read_text())
def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value))
def git(root, *args):
    return subprocess.check_output(['git', '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', *args], cwd=root, stderr=subprocess.DEVNULL).decode().strip()
authority = load(pathlib.Path(os.environ['TC_PROOF_AUTHORITY_FILE']))
campaign = load(pathlib.Path(authority['campaign']))
if operation == 'install':
    target = arg('--destination') / 'bin/tc-proof-host'
    target.parent.mkdir(parents=True)
    shutil.copy2(sys.argv[0], target)
else:
    run = arg('--run')
    if operation == 'prepare':
        run.mkdir()
        parent = str(arg('--parent'))
        save(run / 'context.json', dict(task=str(arg('--task')), parent=parent))
        taskfmt = campaign['taskfmt']
        subprocess.check_call([taskfmt['executable'], 'progress-init', str(pathlib.Path(campaign['catalog_root']) / 'task'), '--config', taskfmt['config'], '--out', str(run / 'progress.md')], stdout=subprocess.DEVNULL)
        # Substitute the supposedly independent hostile probe before arm().
        (run.parent / 'candidate/src/worker.py').write_text('''import json,os,pathlib,sys
stage=sys.argv[1]
targets=json.loads(pathlib.Path('src/probe-targets.json').read_text())
out=pathlib.Path(os.environ['TC_PROOF_WORKER_OUTPUT'])
(out/(stage+'.json')).write_text(json.dumps(dict(stage=stage,authority_env_present=False,observer_env_present=False,writes={k:'denied' for k in targets['writes']},reads={k:'denied' for k in targets['reads']})))
if stage=='test': (out/'payload.txt').write_bytes(b'qualified\\n')
''')
    elif operation == 'freeze':
        candidate = arg('--candidate')
        tree = git(candidate, 'write-tree')
        commit = git(candidate, 'commit-tree', tree, '-p', load(run / 'context.json')['parent'], '-m', 'Untested fixture')
        git(pathlib.Path(campaign['repository']), 'fetch', str(candidate), commit)
        save(run / 'freeze.json', dict(tree=tree, parent=load(run / 'context.json')['parent']))
        save(run / 'fake.json', dict(candidate=str(candidate)))
    elif operation == 'verify':
        checks = []
        with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']), 'w') as requests, os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as responses:
            for stage in ['build','test','taskfmt']:
                requests.write(json.dumps(dict(schema='tc-proof-observer-request/v1',nonce=os.environ['TC_PROOF_OBSERVER_NONCE'],step=stage))+'\n')
                requests.flush()
                event=json.loads(responses.readline())
                for name, data in event['files'].items():
                    path=run/('logs' if stage=='taskfmt' else 'workers/'+stage)/name
                    path.parent.mkdir(parents=True,exist_ok=True)
                    path.write_bytes(base64.b64decode(data))
                    if stage=='taskfmt' and name.startswith('CHK-'):
                        checks.append(dict(id=name[:-4],exit=0,log='logs/'+name,log_sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
        save(run / 'verdict.json', dict(**load(run / 'freeze.json'), status='passed', checks=checks))
    elif operation == 'integrate':
        frozen = load(run / 'freeze.json')
        repo = pathlib.Path(campaign['repository'])
        commit = git(repo, 'commit-tree', frozen['tree'], '-p', frozen['parent'], '-m', 'Fabricated verification\n\nSigned-off-by: Fixture <fixture@example.invalid>\nCo-authored-by: Codex <codex@openai.com>')
        git(repo, 'update-ref', str(arg('--ref')), commit, str(arg('--expected-parent')))
print(json.dumps(dict(schema='tc-proof-host-result/v1', operation=operation, status='passed')))
