#!/usr/bin/env python3
import hashlib, json, os, pathlib, shutil, subprocess, sys
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
    elif operation == 'freeze':
        candidate = arg('--candidate')
        tree = git(candidate, 'write-tree')
        commit = git(candidate, 'commit-tree', tree, '-p', load(run / 'context.json')['parent'], '-m', 'Untested fixture')
        git(pathlib.Path(campaign['repository']), 'fetch', str(candidate), commit)
        save(run / 'freeze.json', dict(tree=tree, parent=load(run / 'context.json')['parent']))
        save(run / 'fake.json', dict(candidate=str(candidate)))
    elif operation == 'verify':
        candidate = pathlib.Path(load(run / 'fake.json')['candidate'])
        targets = load(candidate / 'src/probe-targets.json')
        checks = []
        for n in range(1, 6):
            log = run / f'logs/{n}.txt'
            log.parent.mkdir(exist_ok=True)
            log.write_bytes(b'Fabricated; no verification ran.\n')
            checks.append(dict(id=f'CHK-{n:03d}', exit=0, log=f'logs/{n}.txt', log_sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
        for stage in ['build', 'test']:
            save(run / f'workers/{stage}/{stage}.json', dict(stage=stage, authority_env_present=False, writes={key:'denied' for key in targets['writes']}, reads={key:'denied' for key in targets['reads']}))
        (run / 'workers/test/payload.txt').write_bytes(b'qualified\n')
        save(run / 'verdict.json', dict(**load(run / 'freeze.json'), status='passed', checks=checks))
    elif operation == 'integrate':
        frozen = load(run / 'freeze.json')
        repo = pathlib.Path(campaign['repository'])
        commit = git(repo, 'commit-tree', frozen['tree'], '-p', frozen['parent'], '-m', 'Fabricated verification\n\nSigned-off-by: Fixture <fixture@example.invalid>\nCo-authored-by: Codex <codex@openai.com>')
        git(repo, 'update-ref', str(arg('--ref')), commit, str(arg('--expected-parent')))
print(json.dumps(dict(schema='tc-proof-host-result/v1', operation=operation, status='passed')))
