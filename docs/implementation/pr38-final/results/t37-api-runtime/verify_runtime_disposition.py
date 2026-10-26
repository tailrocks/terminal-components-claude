import collections
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
INTEGRATION = Path('/Users/donbeave/Downloads/terminal-components-claude')
API_RULES = ROOT / 'docs/implementation/pr38-final/API-RULES.md'
OUTPUT = ROOT / 'docs/implementation/pr38-final/results/t37-api-runtime/foundation-runtime-disposition.json'
CROSSWALK = ROOT / 'docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json'
RAW_COMMIT = '2fee2c277d1213fc5fad5d81eb77479600182b69'
SOURCE_COMMIT = '1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
CLEANUP_COMMIT = '7368d4fc5a71e46cae5ee92963bcf3a6943953e9'
CLEANUP_EVIDENCE_COMMIT = '2fee2c277d1213fc5fad5d81eb77479600182b69'


def git_blob(commit: str, path: str) -> bytes:
    return subprocess.check_output(['git', '-C', str(INTEGRATION), 'show', f'{commit}:{path}'])


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def normalize(text: str) -> str:
    return re.sub(r'\s+', ' ', text).strip()


def mask_rust_noncode(source: str) -> str:
    chars = list(source)
    i = 0

    def blank(start: int, end: int) -> None:
        for pos in range(start, end):
            if chars[pos] not in '\r\n':
                chars[pos] = ' '

    while i < len(source):
        if source.startswith('//', i):
            end = source.find('\n', i)
            if end < 0:
                end = len(source)
            blank(i, end)
            i = end
            continue
        if source.startswith('/*', i):
            start = i
            i += 2
            depth = 1
            while i < len(source) and depth:
                if source.startswith('/*', i):
                    depth += 1
                    i += 2
                elif source.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            blank(start, i)
            continue
        raw = re.match(r'(?:br|r)(#+)?"', source[i:])
        if raw:
            start = i
            hashes = raw.group(1) or ''
            i += raw.end()
            closing = '"' + hashes
            end = source.find(closing, i)
            i = len(source) if end < 0 else end + len(closing)
            blank(start, i)
            continue
        if source[i] in 'b' and i + 1 < len(source) and source[i + 1] == '"':
            quote = i + 1
        elif source[i] == '"':
            quote = i
        else:
            quote = -1
        if quote >= 0:
            start = i
            i = quote + 1
            while i < len(source):
                if source[i] == '\\':
                    i += 2
                elif source[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            blank(start, i)
            continue
        if source[i] == "'":
            end = i + 1
            escaped = False
            while end < len(source) and source[end] not in '\r\n' and end - i <= 13:
                if source[end] == "'" and not escaped:
                    end += 1
                    blank(i, end)
                    i = end
                    break
                if source[end] == '\\' and not escaped:
                    escaped = True
                else:
                    escaped = False
                end += 1
            else:
                i += 1
            continue
        i += 1
    return ''.join(chars)


def line_depth(source: str, line_number: int) -> int:
    depth = 0
    lines = mask_rust_noncode(source).splitlines()
    for line in lines[:line_number - 1]:
        depth += line.count('{') - line.count('}')
    return depth


def slug(title: str) -> str:
    title = title.lower()
    title = re.sub(r'[^a-z0-9 -]', '', title)
    return re.sub(r'\s+', '-', title.strip())


def fail(message: str) -> None:
    raise SystemExit(message)


def verify_scope_attachment(attachment: dict, path: str, source_path: str, cleanup_sources: dict) -> None:
    scope = attachment.get('scope_evidence')
    if not scope:
        return
    if scope['kind'] == 'module_level_function':
        evidence_path = scope['path']
        if evidence_path != path:
            fail(f'function scope path mismatch for {attachment["symbol"]}')
        if evidence_path not in cleanup_sources:
            cleanup_sources[evidence_path] = git_blob(CLEANUP_COMMIT, evidence_path).decode()
        source = cleanup_sources[evidence_path]
        line_number = scope['line_number']
        lines = source.splitlines()
        if sum(line.strip() == attachment['declaration_line'] for line in lines) != 1:
            fail(f'function declaration is not unique for {attachment["symbol"]}')
        if line_number < 1 or line_number > len(lines) or lines[line_number - 1].strip() != attachment['declaration_line']:
            fail(f'function declaration line mismatch for {attachment["symbol"]}')
        if line_depth(source, line_number) != 0 or scope['lexical_parent'] != 'module':
            fail(f'function is not attached at module scope: {attachment["symbol"]}')
        if not re.search(r'\bfn\s+' + re.escape(attachment['symbol']) + r'\b', lines[line_number - 1]):
            fail(f'function symbol mismatch for {attachment["symbol"]}')
        module_leaf = scope['module'].split('::')[-1]
        source_leaf = Path(source_path).stem if Path(source_path).name != 'mod.rs' else Path(source_path).parent.name
        if module_leaf != source_leaf:
            fail(f'function module does not match source path for {attachment["symbol"]}')
        return
    if scope['kind'] == 'module_documentation':
        module = scope['module']
        source_leaf = Path(source_path).stem if Path(source_path).name != 'mod.rs' else Path(source_path).parent.name
        if module.split('::')[-1] != source_leaf or attachment['symbol'] != module:
            fail(f'module documentation identity mismatch for {module}')
        prior_parts = None
        for declaration in scope['module_declarations']:
            evidence_path = declaration['path']
            if evidence_path not in cleanup_sources:
                cleanup_sources[evidence_path] = git_blob(CLEANUP_COMMIT, evidence_path).decode()
            source = cleanup_sources[evidence_path]
            lines = source.splitlines()
            line_number = declaration['line_number']
            if sum(line.strip() == declaration['declaration_line'] for line in lines) != 1:
                fail(f'module declaration is not unique for {declaration["module"]}')
            if line_number < 1 or line_number > len(lines) or lines[line_number - 1].strip() != declaration['declaration_line']:
                fail(f'module declaration line mismatch for {declaration["module"]}')
            if line_depth(source, line_number) != 0 or declaration['scope'] != 'module':
                fail(f'module declaration is not at module scope: {declaration["module"]}')
            match = re.search(r'\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', declaration['declaration_line'])
            parts = declaration['module'].split('::')
            if not match or match.group(1) != parts[-1]:
                fail(f'module declaration name mismatch for {declaration["module"]}')
            if prior_parts is not None and parts[:-1] != prior_parts:
                fail(f'module declaration chain is discontinuous for {declaration["module"]}')
            prior_parts = parts
        if not scope['module_declarations'] or prior_parts != module.split('::'):
            fail(f'module declaration chain does not reach {module}')
        return
    fail(f'unknown lexical scope evidence kind: {scope["kind"]}')


def main() -> None:
    result = json.loads(OUTPUT.read_text())
    crosswalk_bytes = CROSSWALK.read_bytes()
    if sha256(API_RULES.read_bytes()) != result['api_rules_document']['sha256']:
        fail('candidate API-RULES SHA does not match the recorded dependency')
    if sha256(crosswalk_bytes) != result['identity_crosswalk']['sha256']:
        fail('identity crosswalk SHA does not match the recorded dependency')

    raw_bytes = git_blob(RAW_COMMIT, result['raw_source_inventory']['path'])
    if sha256(raw_bytes) != result['raw_source_inventory']['sha256']:
        fail('raw source inventory SHA does not match the recorded dependency')
    raw = json.loads(raw_bytes)
    if raw['record_count'] != result['raw_source_inventory']['record_count']:
        fail('raw source inventory record count changed')

    cleanup_map_bytes = {}
    for key, row in result['cleanup_maps'].items():
        blob = git_blob(row['commit'], row['path'])
        if sha256(blob) != row['sha256']:
            fail(f'{key} SHA does not match the recorded dependency')
        cleanup_map_bytes[key] = blob
    removal = json.loads(cleanup_map_bytes['source_removal'])
    removed_targets = {row['path']: row for row in removal['removed_test_targets']}
    cfg_scopes = removal['removed_cfg_test_scopes']

    crosswalk = json.loads(crosswalk_bytes)
    cohort = [
        row for row in crosswalk['records']
        if row['path'].startswith(('crates/termrock-runtime/', 'crates/termrock-session/'))
    ]
    records = result['records']
    if len(cohort) != 304 or len(records) != 304:
        fail(f'expected 304 cohort rows; crosswalk={len(cohort)} output={len(records)}')
    if len({row['id'] for row in records}) != 304:
        fail('output IDs are not unique')
    if len({row['source_map_record'] for row in records}) != 304:
        fail('output source_map_record values are not unique')

    docs: dict[str, str] = {}
    allowed_docs = {
        'docs/contracts/foundation-runtime.md',
        'docs/contracts/authoring.md',
        'docs/contracts/theming.md',
        'docs/contracts/safety-invariants.md',
    }
    cleanup_sources: dict[str, str | None] = {}
    source_bytes: dict[str, bytes] = {}
    disposition_counts = collections.Counter()
    attachment_counts = collections.Counter()

    for output_row, source_row in zip(records, cohort):
        identity_keys = ('id', 'source_map_record', 'path', 'line_range', 'source_sha256', 'text_sha256')
        for key in identity_keys:
            if output_row.get(key) != source_row.get(key):
                fail(f'identity mismatch for source_map_record {source_row["source_map_record"]}: {key}')

        path = source_row['path']
        raw_row = raw['records'][source_row['source_map_record']]
        if (raw_row['path'] != path or raw_row['start_line'] != source_row['line_range']['start']
                or raw_row['end_line'] != source_row['line_range']['end']
                or raw_row['source_sha256'] != source_row['source_sha256']):
            fail(f'raw source identity mismatch for source_map_record {source_row["source_map_record"]}')
        if path not in source_bytes:
            source_bytes[path] = git_blob(SOURCE_COMMIT, path)
        if sha256(source_bytes[path]) != source_row['source_sha256']:
            fail(f'frozen source SHA mismatch for {path}')

        attachment = output_row['cleanup_attachment']
        attachment_counts[attachment['status']] += 1
        if attachment['source_commit'] != CLEANUP_COMMIT:
            fail(f'wrong cleanup commit for source_map_record {source_row["source_map_record"]}')
        if attachment['status'] == 'retained_production_item':
            scope = attachment.get('scope_evidence')
            evidence_path = scope['path'] if scope and scope['kind'] == 'module_level_function' else path
            if evidence_path not in cleanup_sources:
                cleanup_sources[evidence_path] = git_blob(CLEANUP_COMMIT, evidence_path).decode()
            if scope and scope['kind'] == 'module_documentation':
                evidence_path = scope['module_declarations'][-1]['path']
                if evidence_path not in cleanup_sources:
                    cleanup_sources[evidence_path] = git_blob(CLEANUP_COMMIT, evidence_path).decode()
                declaration = scope['module_declarations'][-1]['declaration_line']
                if declaration not in [line.strip() for line in cleanup_sources[evidence_path].splitlines()]:
                    fail(f'module attachment declaration missing after cleanup: {scope["module"]}')
            elif attachment['declaration_line'] not in [line.strip() for line in cleanup_sources[evidence_path].splitlines()]:
                fail(f'attachment declaration missing after cleanup: {path}:{attachment["symbol"]}')
            verify_scope_attachment(attachment, path, path, cleanup_sources)
        elif attachment['status'] == 'removed_test_target':
            if path not in removed_targets:
                fail(f'missing removed-target map for {path}')
        elif attachment['status'] == 'removed_cfg_test_helper':
            if not any(scope['path'] == path for scope in cfg_scopes):
                fail(f'missing cfg(test) removal scope for {path}')
            if path not in cleanup_sources:
                cleanup_sources[path] = git_blob(CLEANUP_COMMIT, path).decode()
            if re.search(r'\b' + re.escape(attachment['symbol']) + r'\b', cleanup_sources[path]):
                fail(f'retired cfg(test) helper remains after cleanup: {attachment["symbol"]}')
        else:
            fail(f'unknown cleanup attachment status: {attachment["status"]}')

        disposition = output_row['semantic_disposition']
        disposition_counts[disposition] += 1
        destination = output_row['destination']
        excerpt = output_row['supporting_excerpt']
        if destination:
            doc_path, anchor = destination.split('#', 1)
            if doc_path not in allowed_docs:
                fail(f'destination is outside maintained docs: {destination}')
            if doc_path not in docs:
                docs[doc_path] = (ROOT / doc_path).read_text()
            headings = {slug(value) for value in re.findall(r'^#{1,6}\s+(.+?)\s*#*\s*$', docs[doc_path], re.M)}
            if anchor not in headings:
                fail(f'missing destination anchor: {destination}')
            if not excerpt or normalize(excerpt) not in normalize(docs[doc_path]):
                fail(f'missing exact supporting excerpt: {destination}, source_map_record={source_row["source_map_record"]}')
            if not output_row['required_clauses'] or not output_row['coverage_rationale']:
                fail(f'mapped row lacks required clauses or rationale: {source_row["source_map_record"]}')
        elif not output_row.get('specific_reason'):
            fail(f'unmapped row lacks a group-specific reason: {source_row["source_map_record"]}')

    expected_attachments = collections.Counter({
        'retained_production_item': 299,
        'removed_test_target': 3,
        'removed_cfg_test_helper': 2,
    })
    if attachment_counts != expected_attachments:
        fail(f'cleanup attachment status counts differ: {attachment_counts}')
    if result['selected_test_cases'] != 0 or result['runtime_tests_executed'] or result['behavior_status'] != 'NOT VERIFIED':
        fail('T37 behavior evidence must remain zero-case and NOT VERIFIED')

    print('PASS: 304 source identities and hashes match the crosswalk.')
    print('PASS: imported candidate API-RULES path and frozen cleanup attachments match recorded commits.')
    print('PASS: corrected free-function and module-comment attachments match their lexical scopes and declaration chains.')
    print('PASS: mapped rows resolve to exact Markdown anchors and excerpts; other rows have reasons.')
    print('PASS: selected behavior cases=0; runtime behavior=NOT VERIFIED.')
    print('NOTE: semantic coverage requires the independent Sol high review; this verifier checks structure only.')
    print(f'Counts: attachments={dict(attachment_counts)} dispositions={dict(disposition_counts)}')


if __name__ == '__main__':
    main()
