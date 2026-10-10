#!/usr/bin/env python3
"""Capture one bounded PR17 provider/ref observation through the pinned Rust capturer."""
import base64
import hashlib
import json
import os
import pathlib
import subprocess
import sys
from datetime import datetime, timezone

ROOT = pathlib.Path('/private/tmp/termrock-vis01-pr17-provider-current-observation-r1-20261010')
CAPTURE = pathlib.Path('/private/tmp/termrock-qualified-capture-command-58de6bb-r1-20261010/capture-command')
CAPTURE_SHA256 = '5564b7749faba78d18cc060dd8e34f4d753ae6c5cbf55457d4a5e220a84f93e8'
CAPTURE_SOURCE_COMMIT = '58de6bb6c8e23da8a1f66237db179d7c58c79b28'
CAPTURE_SOURCE_PATH = 'crates/termrock-visibility-tests/src/bin/capture-command.rs'
CAPTURE_SOURCE_SHA256 = 'de1b9211b8671b28b57ffe288be37fc58f10e43a836bfd9153813704e0dfdda7'
CAPTURE_SOURCE_GIT_BLOB = 'fabaad307970cda5d910a008b8f22f2c7186531b'
CAPTURE_COPY_RECEIPT = '/private/tmp/termrock-qualified-capture-command-58de6bb-r1-20261010/copy-receipt.json'
CAPTURE_COPY_RECEIPT_SHA256 = 'b49074a6b7ec6b1aca69d5c35aca824143202ef0fee61ad4d61e7c9f548ff058'
GH = pathlib.Path('/opt/homebrew/bin/gh')
GH_SHA256 = '81904d0aaa9fb3d9e35e13e7b1fe917451df0430eaee63144c69426d5d2832f1'
REPO = 'tailrocks/terminal-components-claude'
BASE_ENV = {
    'GH_PAGER': 'cat',
    'GH_PROMPT_DISABLED': '1',
    'HOME': '/Users/donbeave',
    'NO_COLOR': '1',
    'PATH': '/opt/homebrew/bin:/usr/bin:/bin',
}
CAPTURE_ENV = {'HOME': '/Users/donbeave', 'PATH': '/usr/bin:/bin'}
os.umask(0o077)


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def write_exclusive(path, data):
    path = pathlib.Path(path)
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    with open(path, 'xb') as f:
        f.write(data)

if not ROOT.is_dir() or ROOT.is_symlink():
    raise SystemExit('output root is not the expected real directory')
if sha256_file(CAPTURE) != CAPTURE_SHA256:
    raise SystemExit('capture-command binary pin mismatch')
if sha256_file(GH) != GH_SHA256:
    raise SystemExit('gh executable pin mismatch')

records = []
failed_capture = False

def capture(label, argv, tolerate_child_exit=False):
    global failed_capture
    req_dir = ROOT / 'requests'
    raw_dir = ROOT / 'raw'
    wrap_dir = ROOT / 'wrapper'
    for d in (req_dir, raw_dir, wrap_dir):
        d.mkdir(mode=0o700, exist_ok=True)
    request = {
        'schema': 'termrock-command-capture-request/v1',
        'argv': [str(x) for x in argv],
        'cwd': '/',
        'env': BASE_ENV,
        'timeout_ms': 90000,
        'stdout_path': str(raw_dir / f'{label}.stdout.raw'),
        'stderr_path': str(raw_dir / f'{label}.stderr.raw'),
        'result_path': str(raw_dir / f'{label}.result.json'),
    }
    request_bytes = (json.dumps(request, indent=2, sort_keys=True) + '\n').encode()
    request_path = req_dir / f'{label}.request.json'
    write_exclusive(request_path, request_bytes)
    wrapped = subprocess.run(
        [str(CAPTURE), '--request', str(request_path)],
        cwd='/', env=CAPTURE_ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        check=False,
    )
    wrap_out_path = wrap_dir / f'{label}.stdout.raw'
    wrap_err_path = wrap_dir / f'{label}.stderr.raw'
    write_exclusive(wrap_out_path, wrapped.stdout)
    write_exclusive(wrap_err_path, wrapped.stderr)
    result_path = pathlib.Path(request['result_path'])
    result_doc = None
    if result_path.is_file():
        try:
            result_doc = json.loads(result_path.read_bytes())
        except Exception:
            pass
    row = {
        'label': label,
        'argv': request['argv'],
        'request_path': str(request_path),
        'request_bytes': len(request_bytes),
        'request_sha256': sha256_bytes(request_bytes),
        'capture_tool_argv': [str(CAPTURE), '--request', str(request_path)],
        'capture_tool_exit_code': wrapped.returncode,
        'capture_tool_stdout_path': str(wrap_out_path),
        'capture_tool_stdout_bytes': len(wrapped.stdout),
        'capture_tool_stdout_sha256': sha256_bytes(wrapped.stdout),
        'capture_tool_stderr_path': str(wrap_err_path),
        'capture_tool_stderr_bytes': len(wrapped.stderr),
        'capture_tool_stderr_sha256': sha256_bytes(wrapped.stderr),
        'raw_stdout_path': request['stdout_path'],
        'raw_stderr_path': request['stderr_path'],
        'raw_result_path': request['result_path'],
        'capture_result': result_doc,
    }
    for key, path_key in [('raw_stdout', 'stdout_path'), ('raw_stderr', 'stderr_path'), ('raw_result', 'result_path')]:
        p = pathlib.Path(request[path_key])
        if p.is_file():
            data = p.read_bytes()
            row[key + '_bytes'] = len(data)
            row[key + '_sha256'] = sha256_bytes(data)
        else:
            row[key + '_missing'] = True
    complete = (
        wrapped.returncode == 0 and result_doc is not None
        and result_doc.get('outcome') == 'complete'
        and result_doc.get('result_file_written') is not False
    )
    child_exit = None if result_doc is None else result_doc.get('child', {}).get('exit_code')
    row['capture_wrapper_complete'] = complete
    row['child_exit_code'] = child_exit
    row['child_exit_expected_nonzero_allowed'] = tolerate_child_exit
    records.append(row)
    if not complete or (child_exit not in (0, None) and not tolerate_child_exit):
        failed_capture = True
    if result_doc is None or child_exit not in (0, None):
        return b''
    out_path = pathlib.Path(request['stdout_path'])
    return out_path.read_bytes() if out_path.is_file() else b''

repo = REPO
# Discovery and pre-capture readback.
pre_candidate = capture('pre-candidate-ref', [GH, 'api', f'repos/{repo}/git/ref/heads/termrock-implementation'])
pre_reference = capture('pre-reference-ref', [GH, 'api', f'repos/{repo}/git/ref/heads/visual-baseline'])
pre_tag = capture('pre-tag-ref', [GH, 'api', f'repos/{repo}/git/ref/tags/visual-baseline'])
pr_bytes = capture('pr-17', [GH, 'pr', 'view', '17', '--repo', repo, '--json', 'number,state,isDraft,headRefName,headRefOid,baseRefName,mergeStateStatus,statusCheckRollup'])
version_bytes = capture('gh-version', [GH, '--version'])

candidate_sha = None
reference_sha = None
tag_object_sha = None
pr_doc = None
try:
    candidate_sha = json.loads(pre_candidate)['object']['sha']
except Exception:
    pass
try:
    reference_sha = json.loads(pre_reference)['object']['sha']
except Exception:
    pass
try:
    tag_object_sha = json.loads(pre_tag)['object']['sha']
except Exception:
    pass
try:
    pr_doc = json.loads(pr_bytes)
except Exception:
    pass

list_bytes = b''
selected_run = None
if candidate_sha:
    list_bytes = capture('runs-for-candidate', [GH, 'run', 'list', '--repo', repo, '--commit', candidate_sha, '--limit', '20', '--json', 'databaseId,headSha,event,status,conclusion,createdAt,updatedAt,url,workflowName,headBranch'])
try:
    listed = json.loads(list_bytes) if list_bytes else []
    matches = [row for row in listed if row.get('headSha') == candidate_sha]
    if matches:
        selected_run = matches[0]
except Exception:
    selected_run = None

run_doc = None
workflow_doc = None
check_suites_doc = None
if selected_run is not None:
    run_id = selected_run.get('databaseId') or selected_run.get('id')
    run_bytes = capture('run', [GH, 'api', f'repos/{repo}/actions/runs/{run_id}'])
    try:
        run_doc = json.loads(run_bytes)
    except Exception:
        run_doc = None
    workflow_id = None if run_doc is None else run_doc.get('workflow_id')
    check_suite_id = None if run_doc is None else run_doc.get('check_suite_id')
    if workflow_id is not None:
        workflow_bytes = capture('workflow', [GH, 'api', f'repos/{repo}/actions/workflows/{workflow_id}'])
        try:
            workflow_doc = json.loads(workflow_bytes)
        except Exception:
            workflow_doc = None
    check_suites_bytes = capture('check-suites', [GH, 'api', f'repos/{repo}/commits/{candidate_sha}/check-suites?per_page=100'])
    try:
        check_suites_doc = json.loads(check_suites_bytes)
    except Exception:
        check_suites_doc = None
    if check_suite_id is not None:
        capture('suite-check-runs', [GH, 'api', f'repos/{repo}/check-suites/{check_suite_id}/check-runs?per_page=100'])
    capture('check-runs', [GH, 'api', f'repos/{repo}/commits/{candidate_sha}/check-runs?per_page=100'])
    capture('jobs', [GH, 'api', f'repos/{repo}/actions/runs/{run_id}/jobs?per_page=100'])
    capture('artifacts', [GH, 'api', f'repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100'])
    logs = capture('run-logs-combined', [GH, 'api', '--include', f'repos/{repo}/actions/runs/{run_id}/logs'], tolerate_child_exit=True)
    separator = b'\r\n\r\n'
    split = logs.find(separator)
    sep_len = len(separator)
    if split < 0:
        separator = b'\n\n'; split = logs.find(separator); sep_len = len(separator)
    if split >= 0:
        write_exclusive(ROOT / 'raw' / 'run-logs.headers', logs[:split + sep_len])
        write_exclusive(ROOT / 'raw' / 'run-logs.body', logs[split + sep_len:])
    # Capture the candidate workflow bytes independently from provider conclusions.
    content_bytes = capture('workflow-source-api', [GH, 'api', f'repos/{repo}/contents/.github/workflows/ci.yml?ref={candidate_sha}'])
    try:
        content_doc = json.loads(content_bytes)
        if content_doc.get('encoding') == 'base64' and isinstance(content_doc.get('content'), str):
            workflow_source = base64.b64decode(content_doc['content'], validate=False)
            write_exclusive(ROOT / 'raw' / 'source' / 'ci.yml', workflow_source)
            content_doc_summary = {
                'path': '.github/workflows/ci.yml',
                'candidate_commit': candidate_sha,
                'api_blob_sha': content_doc.get('sha'),
                'api_size': content_doc.get('size'),
                'decoded_bytes': len(workflow_source),
                'decoded_sha256': sha256_bytes(workflow_source),
                'git_blob_sha1': hashlib.sha1(f'blob {len(workflow_source)}\0'.encode() + workflow_source).hexdigest(),
                'content_matches_git_blob_sha': hashlib.sha1(f'blob {len(workflow_source)}\0'.encode() + workflow_source).hexdigest() == content_doc.get('sha'),
            }
            write_exclusive(ROOT / 'raw' / 'source' / 'ci-source-metadata.json', (json.dumps(content_doc_summary, indent=2, sort_keys=True) + '\n').encode())
    except Exception as exc:
        write_exclusive(ROOT / 'raw' / 'source' / 'api-decode-error.json', (json.dumps({'error_type': type(exc).__name__}, sort_keys=True) + '\n').encode())

# Post-capture remote ref readback, never assumed to match the pre-capture state.
post_candidate = capture('post-candidate-ref', [GH, 'api', f'repos/{repo}/git/ref/heads/termrock-implementation'])
post_reference = capture('post-reference-ref', [GH, 'api', f'repos/{repo}/git/ref/heads/visual-baseline'])
post_tag = capture('post-tag-ref', [GH, 'api', f'repos/{repo}/git/ref/tags/visual-baseline'])
post_tag_target = None
try:
    post_tag_object = json.loads(post_tag)
    tag_sha = post_tag_object['object']['sha']
    post_tag_target = capture('post-tag-object', [GH, 'api', f'repos/{repo}/git/tags/{tag_sha}'])
except Exception:
    pass

post_candidate_sha = post_reference_sha = post_tag_sha = None
try: post_candidate_sha = json.loads(post_candidate)['object']['sha']
except Exception: pass
try: post_reference_sha = json.loads(post_reference)['object']['sha']
except Exception: pass
try: post_tag_sha = json.loads(post_tag)['object']['sha']
except Exception: pass

def sha256_path(path):
    return sha256_file(path) if pathlib.Path(path).is_file() else None

summary = {
    'schema': 'termrock-pr17-provider-observation-capture/v1',
    'repository': repo,
    'capture_started_at_utc': records[0]['capture_result'].get('started_at') if records and records[0].get('capture_result') else None,
    'capture_ended_at_utc': datetime.now(timezone.utc).isoformat(),
    'classification': 'RAW_CAPTURE_NOT_NORMALIZED; whether CURRENT depends on exact pre/post head identity and later report-source pointer.',
    'capture_tool': {
        'path': str(CAPTURE), 'sha256': CAPTURE_SHA256,
        'source_commit': CAPTURE_SOURCE_COMMIT, 'source_path': CAPTURE_SOURCE_PATH,
        'source_git_blob': CAPTURE_SOURCE_GIT_BLOB, 'source_sha256': CAPTURE_SOURCE_SHA256,
        'copy_receipt_path': str(CAPTURE_COPY_RECEIPT),
        'copy_receipt_sha256': CAPTURE_COPY_RECEIPT_SHA256,
        'qualification_claim': 'identity/source hashes are recorded; no qualification claim is made by this packet alone',
    },
    'orchestrator': {
        'path': str(pathlib.Path(__file__).resolve()),
        'sha256': sha256_file(__file__), 'bytes': pathlib.Path(__file__).stat().st_size,
        'python_executable': sys.executable, 'python_version': sys.version,
    },
    'gh_tool': {'path': str(GH), 'sha256': GH_SHA256, 'version_stdout_sha256': sha256_bytes(version_bytes)},
    'pre_capture_readback': {
        'candidate_commit': candidate_sha, 'reference_commit': reference_sha,
        'tag_object': tag_object_sha, 'pr_head': None if pr_doc is None else pr_doc.get('headRefOid'),
        'pr_state': None if pr_doc is None else pr_doc.get('state'),
        'pr_draft': None if pr_doc is None else pr_doc.get('isDraft'),
        'pr_merge_state': None if pr_doc is None else pr_doc.get('mergeStateStatus'),
    },
    'selected_run': selected_run,
    'provider_detail': None if run_doc is None else {
        'run_id': run_doc.get('id'), 'head_sha': run_doc.get('head_sha'),
        'status': run_doc.get('status'), 'conclusion': run_doc.get('conclusion'),
        'event': run_doc.get('event'), 'run_attempt': run_doc.get('run_attempt'),
        'workflow_id': run_doc.get('workflow_id'), 'check_suite_id': run_doc.get('check_suite_id'),
    },
    'post_capture_readback': {
        'candidate_commit': post_candidate_sha, 'reference_commit': post_reference_sha,
        'tag_object': post_tag_sha,
        'tag_target_commit': None if not post_tag_target else (lambda d: d.get('object', {}).get('sha'))(json.loads(post_tag_target)),
    },
    'readback_stable': {
        'candidate': candidate_sha is not None and candidate_sha == post_candidate_sha,
        'reference': reference_sha is not None and reference_sha == post_reference_sha,
        'tag': tag_object_sha is not None and tag_object_sha == post_tag_sha,
        'pr_head_matches_candidate': pr_doc is not None and candidate_sha == pr_doc.get('headRefOid'),
        'run_head_matches_candidate': run_doc is not None and candidate_sha == run_doc.get('head_sha'),
    },
    'workflow_source': None,
    'commands': records,
    'failure_cause': 'NOT_INFERRED',
    'test_or_product_execution': 'NOT_RUN_BY_THIS_CAPTURE',
}
source_meta = ROOT / 'raw' / 'source' / 'ci-source-metadata.json'
if source_meta.is_file(): summary['workflow_source'] = json.loads(source_meta.read_bytes())
summary_path = ROOT / 'capture-summary.json'
write_exclusive(summary_path, (json.dumps(summary, indent=2, sort_keys=True) + '\n').encode())

members = []
for path in sorted(ROOT.rglob('*')):
    if path.is_file() and path.name != 'MANIFEST.json':
        data = path.read_bytes()
        members.append({'path': str(path.relative_to(ROOT)), 'bytes': len(data), 'sha256': sha256_bytes(data), 'mode': format(path.stat().st_mode & 0o777, '04o')})
manifest = {
    'schema': 'termrock-pr17-provider-observation-raw-manifest/v1',
    'root': str(ROOT), 'member_count': len(members), 'total_bytes': sum(m['bytes'] for m in members),
    'files': members,
}
manifest_path = ROOT / 'MANIFEST.json'
write_exclusive(manifest_path, (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode())
print(json.dumps({
    'root': str(ROOT), 'manifest_path': str(manifest_path), 'manifest_sha256': sha256_file(manifest_path),
    'capture_summary_sha256': sha256_file(summary_path), 'script_sha256': sha256_file(__file__),
    'pre': summary['pre_capture_readback'], 'post': summary['post_capture_readback'],
    'run': summary['provider_detail'], 'readback_stable': summary['readback_stable'],
    'command_count': len(records), 'failed_capture_wrapper': failed_capture,
}, sort_keys=True))
if failed_capture:
    raise SystemExit('one or more captures did not complete; outputs and manifest preserved')
