import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path.cwd()
INPUT = 'e164e7b5b17e9c2d78bf30cfbbfe6bdb36e7b7ab'
RAW_COMMIT = '2fee2c277d1213fc5fad5d81eb77479600182b69'
PRODUCT = '1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
CLEANUP = '7368d4fc5a71e46cae5ee92963bcf3a6943953e9'
CONTRACTS = 'c817b99e9e43abbbf3519d56b2f5ee86924243d8'
PLAN = Path('/Users/donbeave/Downloads/termrock-pr38-plan')
OUTDIR = ROOT / 'docs/implementation/pr38-final/results/t37-api-values'
GUIDE = ROOT / 'docs/contracts/foundation-values.md'
AUTH = ROOT / 'docs/contracts/authoring.md'


def git_json(commit, path):
    return json.loads(subprocess.check_output(['git', 'show', f'{commit}:{path}']))

raw_doc = git_json(RAW_COMMIT, 'docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json')
crosswalk = git_json(INPUT, 'docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json')
removal = git_json(RAW_COMMIT, 'docs/implementation/pr38-final/results/t37-foundations/source-removal.json')
raw_by_record = {i: x for i, x in enumerate(raw_doc['records'])}
cross_by_record = {x['source_map_record']: x for x in crosswalk['records']}
cohort = [(i, x) for i, x in enumerate(raw_doc['records']) if x['path'].startswith((
    'crates/termrock-core/', 'crates/termrock-layout/', 'crates/termrock-text/'))]

FOUNDATION = 'docs/contracts/foundation-values.md'
AUTH_PATH = 'docs/contracts/authoring.md'
API_RULES = 'API-RULES.md'

# Every listed clause was checked against the complete source group and its attached item.
# This map is a semantic disposition authored for this exact cohort, not a lexical classifier.
specs = {}

def add(ids, *, section, marker, clause, doc='foundation', disposition='required_maintained_clause', decision_id=None, reason=None, table_cell=None):
    for i in ids:
        if i in specs:
            raise ValueError(f'duplicate rule for {i}')
        specs[i] = {
            'section': section, 'marker': marker, 'clause': clause, 'doc': doc,
            'disposition': disposition, 'decision_id': decision_id, 'reason': reason,
            'table_cell': table_cell,
        }

# Intent identity, result semantics, diagnostic meanings, and preserved input decoding.
add([1], section='intent-claims-and-diagnostics', marker='FocusContinuationUnacknowledged', clause='Identify the owner when a settled focus continuation was not acknowledged before bounded retention expired.')
add([2], section='intent-claims-and-diagnostics', marker='InvalidComponentLayout', clause='Identify the component and optional field or child; None denotes a structural layout error.')
add([3], section='intent-claims-and-diagnostics', marker='UnsupportedClippedComponentSlot', clause='Identify a legacy part painter that cannot describe a partially clipped component part, with its owner and part.')
add([4], section='intent-claims-and-diagnostics', marker='Capacity limits retained diagnostic items.', clause='Retain items up to capacity; accumulate dropped counts with saturating arithmetic, without capping the count at capacity.', decision_id='T37-DIAGNOSTICS-01', reason='The source comment says dropped counts obey capacity. The implementation keeps a saturating cumulative counter; the wording may imply a cap, but no source defect is established.')
specs[4]['support_markers'] = ['`Diagnostics::append` keeps existing items first.', 'Capacity limits retained diagnostic items.']
add([12], section='intent-claims-and-diagnostics', marker='For `RoutedInput`', clause='Decode middle-button Down into its routed variant; decode every other event through the established Input decoder unchanged.')
add([13], section='intent-claims-and-diagnostics', marker='A tracked token is opaque', clause='A token identifies one exact queue entry; clones share its single claim and do not keep a cleared or dropped entry pending.')
add([15], section='intent-claims-and-diagnostics', marker='`Claimed` means this call', clause='Claimed means this call claimed the exact pending entry.')
add([16], section='intent-claims-and-diagnostics', marker='`AlreadyClaimed` means', clause='AlreadyClaimed means the exact entry remains queued with an earlier claim.')
add([17], section='intent-claims-and-diagnostics', marker='`NotPending` means', clause='NotPending means the entry was cleared, its queue dropped, or the token belongs to another queue.')
add([18], section='intent-claims-and-diagnostics', marker='A `Mismatch` result', clause='A payload mismatch leaves the exact pending entry and neighboring intents unchanged.')
add([19], section='intent-claims-and-diagnostics', marker='A tracked token is opaque', clause='Cloning the token shares claim identity and does not create another queue entry or permit a second claim.')
add([20], section='intent-claims-and-diagnostics', marker='`claim_next_layer_event` claims', clause='Claim only the next unclaimed event for the owner when its payload matches; leave other owner-bucket intents observable.')
add([21], section='intent-claims-and-diagnostics', marker='A `Mismatch` result', clause='A token claim targets only its exact entry, never a different same-owner/same-payload event; mismatch leaves the entry and neighbors unchanged.')

# Keyboard policy is separated from the session's protocol lifetime and global mirrors.
add([42], section='keyboard-protocol-and-delivery', marker='The core `KeyboardProtocol` value defaults', clause='Keep the core policy default Auto distinct from the session default Off; protocol negotiation remains opt-in.', decision_id='T37-KEY-PROTOCOL-01')
add([55], section='keyboard-list-layout-and-text-rules', marker='| Keyboard protocol |', clause='Auto asks the caller probe before pushing.', doc='authoring', table_cell=1)
add([57], section='keyboard-list-layout-and-text-rules', marker='| Keyboard protocol |', clause='Off writes no protocol bytes.', doc='authoring', table_cell=1)
add([60], section='keyboard-list-layout-and-text-rules', marker='| Keyboard protocol |', clause='The canonical override wins by presence; blank, unknown, and non-UTF-8 values resolve to Auto without legacy fallback.', doc='authoring', table_cell=1)
add([67], section='keyboard-protocol-and-delivery', marker='`ProtocolState::enter` resolves', clause='Resolve Auto through the supplied probe; return a push only when protocol state enables it.')
add([68], section='keyboard-protocol-and-delivery', marker='`leave` returns a pop', clause='Pop once only when this state previously pushed protocol flags.')
add([69], section='keyboard-protocol-and-delivery', marker='`leave` returns a pop', clause='Resume pushes once only when enabled and not already pushed.')
# These legacy process-global APIs have an explicit T20 retirement target.
for i in [70, 71, 72, 73, 74, 75, 76, 87, 88, 89, 91]:
    specs[i] = {
        'section': '7-keep-the-prepared-frame-protocol',
        'marker': "T20 removes core's global protocol and delivery mirrors.",
        'clause': None,
        'doc': 'api_rules',
        'disposition': 'planned_api_retirement',
        'decision_id': 'T20-KEYBOARD-MIRROR-RETIREMENT',
        'reason': 'This source item is a process-global protocol or delivery mirror. The accepted T20 target explicitly removes those core mirrors; preserve it as migration provenance, not a current caller promise.',
        'table_cell': None,
    }
add([77], section='keyboard-protocol-and-delivery', marker='For an Alt-modified key', clause='Learn Super unless refused. Learn Alt only for an Alt-modified key when ALT is 0 and protocol flags are active or its code is outside the listed variants.', disposition='source_wording_conflict', decision_id='T37-KEY-OBSERVE-01', reason='The source comment names protocol-active Alt evidence. The code also learns non-text Alt events without active protocol flags; ALT must be 0. T20 owns the global-to-session transition.')
specs[77]['support_markers'] = ['`observe` learns Super unless delivery was refused.', 'For an Alt-modified key', 'Learning also requires active protocol flags']
add([80], section='keyboard-protocol-and-delivery', marker='`AltKeys::Refused`', clause='Refused permits callers to install stand-ins for unavailable Alt chords.')
add([81], section='keyboard-protocol-and-delivery', marker='`UntilSeen` keeps', clause='UntilSeen keeps macOS Option unconfirmed until an unambiguous Alt chord arrives.')
add([82], section='keyboard-protocol-and-delivery', marker='`WithProtocol` allows', clause='WithProtocol allows Alt only while Kitty protocol flags are active.')
add([84, 85, 86], section='keyboard-protocol-and-delivery', marker='Without a recognized override', clause='Resolve Alt delivery from the override, platform, and named terminal environment values; retain the documented platform fallbacks.')
add([90], section='keyboard-protocol-and-delivery', marker='Legacy literal hints reject', clause='Do not advertise Shift pointer labels; require Super/Alt delivery for matching chord labels, except Alt labels ending in click or drag.')

# Shared list cursor/type-ahead behavior and source-index selection.
add([92], section='list-selection-and-type-ahead', marker='List cursor navigation is an opt-in', clause='List navigation is opt-in; existing widgets retain prior routing until they adopt the shared capability.')
add([103, 104, 105, 106, 107, 108], section='list-selection-and-type-ahead', marker='`Moved::Ignored` means movement', clause='Map Ignored, Stayed, and To to input outcomes; report index only when movement applies and distinguish a type-ahead miss from movement.')
add([113], section='list-selection-and-type-ahead', marker='`Cursor::new(index)` stores', clause='Cursor defaults to skipping disabled rows.')
add([117, 119], section='list-selection-and-type-ahead', marker='`step` bypasses disabled rows', clause='Step skips disabled rows when skip_disabled is true; false permits disabled destinations.')
add([114], section='open-source-wording-conflicts', marker='The `Cursor::new` comment says', clause='Initialize the supplied index unchanged, with vertical axis, no wrapping, and disabled-row skipping; constructor has no row source to validate against.', disposition='source_wording_conflict', decision_id='T37-CURSOR-NEW-01', reason='The source comment says the constructor clamps, but it receives no RowSource. Sol high confirms initialization stores the index unchanged; no source defect is established.')
add([127], section='keyboard-list-layout-and-text-rules', marker='| List selection |', clause='The default routing gives a bound printable key to shortcuts first.', doc='authoring', table_cell=1)
add([130, 131], section='list-selection-and-type-ahead', marker='`Shortcuts` precedence gives', clause='Shortcuts mode gives bound letters to shortcuts first; TypeAhead mode sends every printable character to type-ahead.')
add([137], section='keyboard-list-layout-and-text-rules', marker='| List selection |', clause='A shortcut handler that yields without changing state may receive the same key again for type-ahead.', doc='authoring', table_cell=1)
for i in [135, 136, 143, 144]:
    specs[i] = {
        'section': 'open-source-wording-conflicts',
        'marker': 'Each thread starts with `Shortcuts` precedence',
        'clause': 'Current and install use the calling thread cell; each thread defaults to Shortcuts or Desktop, and installs persist there until changed.',
        'doc': 'foundation',
        'disposition': 'source_wording_conflict',
        'decision_id': 'T37-NAV-STATE-01',
        'reason': 'The named install/current item claims process-wide state, while its static uses thread-local Cell storage. The exact current-thread behavior and comment conflict are recorded; no runtime-instance or cross-thread propagation is promised.',
        'table_cell': None,
        'support_markers': ['`current` and `install` use the calling thread\'s cell.', 'These values are not process-wide'],
    }
add([151, 152, 153, 154, 155], section='list-selection-and-type-ahead', marker='`Moved` maps Ignored, Stayed, and To', clause='Ignored is unhandled, Consumed is handled without visible change, and Changed is handled with change; `or` preserves the strongest in that order.')
add([156, 160, 163, 164], section='list-selection-and-type-ahead', marker='`ClientPlatform::from_env` checks', clause='Use the documented environment override, terminal variables, and build target to resolve the client platform; current caches its first result per process.')
add([165], section='intent-claims-and-diagnostics', marker='`INACTIVE_LAYER` reports', clause='INACTIVE_LAYER marks temporary non-interaction from an inactive owning layer, distinct from configuration-disabled state.')
add([166, 168], section='public-surface-snapshot', marker='| Update, draw, and response |', clause='ERROR denotes a failed operation; INVALID denotes an unusable value; validation failure may carry both.', doc='authoring', table_cell=1)
add([169], section='open-source-wording-conflicts', marker='Range extension starts from the saved selection base', clause='Selection operations use eligibility per operation; saved-base extension can retain disabled base rows and does not globally sanitize selection.', disposition='source_wording_conflict', decision_id='T37-SELECTION-CONTRACT-01', reason='The module comment says disabled rows are never selected. The range comment says prior state stays, while extend rebuilds from saved base and filters current additions; retain operation-specific wording.')
specs[169]['support_markers'] = ['Range extension starts from the saved selection base']
add([181, 191], section='list-selection-and-type-ahead', marker='A click uses Shift', clause='Shift extends, toggle modifiers flip membership, and an unmodified click selects only the clicked row; Shift has priority.')
add([190], section='list-selection-and-type-ahead', marker='On macOS, Option or Cmd', clause='Use Option or Cmd as macOS toggle modifiers; use Ctrl or Cmd on other clients.')
add([192], section='list-selection-and-type-ahead', marker='For key gestures, unmodified Space toggles', clause='Unmodified Space toggles; pass a movement key with Shift and without Ctrl, Alt, or Super to extend.')
add([200], section='list-selection-and-type-ahead', marker='A glyph-only checkbox scope', clause='BoxOnly toggles only when the checkbox glyph is hit; Row scope toggles anywhere in the row.')
add([206], section='list-selection-and-type-ahead', marker='The navigation click policy', clause='The data-grid click policy activates and edits on double click.')
add([207], section='list-selection-and-type-ahead', marker='The navigation click policy', clause='The navigation click policy activates on a single click and never edits.')
add([214, 223], section='keyboard-list-layout-and-text-rules', marker='| List selection |', clause='Store selected rows by source index and supply display order for each gesture.', doc='authoring', table_cell=1)
add([215, 219], section='list-selection-and-type-ahead', marker='Range extension starts from', clause='Rebuild extension from the saved base, excluding its anchor, then add selectable rows in display order; disabled base members may remain while prior range-only members can drop.', disposition='source_wording_conflict', decision_id='T37-SELECTION-CONTRACT-01', reason='This group is a caller-facing selection identity rule. Sol high specifies saved-base reconstruction; the source comment and existing authoring sentence overstate preservation of prior range state.')
add([217], section='list-selection-and-type-ahead', marker='`select_only` clears', clause='Selecting only a disabled row clears the current selection.')
add([218], section='list-selection-and-type-ahead', marker='`select_only` clears', clause='Toggling a disabled row leaves selection unchanged.')
add([224], section='list-selection-and-type-ahead', marker='A context click keeps', clause='Context-clicking a selected row preserves the selection; clicking an enabled row outside it selects only that row; a disabled outside row clears via select_only.')
add([225], section='list-selection-and-type-ahead', marker='`set` changes row membership', clause='Set direct membership without changing the range anchor.')
add([226], section='list-selection-and-type-ahead', marker='`set` changes row membership', clause='Retain selection after filtering and remap after reorder/removal; None drops the row.')
add([227], section='list-selection-and-type-ahead', marker='`set` changes row membership', clause='Remap selected source rows after reorder/removal; drop a row when mapping returns None.')

# Cursor shapes, bounded columns/composition, and split sizing.
add([228], section='layout-and-bounded-work', marker='`ShapeSync` is opt-in.', clause='ShapeSync is an opt-in value API; the frozen source comment reports no production emitter, which is source snapshot context rather than behavior proof.', disposition='source_observation', reason='This module group reports shape adoption status in the frozen source. It does not state a caller failure, identity, or safety rule; keep the statement explicitly as unverified repository context.')
add([253], section='keyboard-list-layout-and-text-rules', marker='| Cursor shape |', clause='Request shapes only when supported, emit only on change, and reset shapes previously sent by the process.', doc='authoring', table_cell=1)
add([256], section='layout-and-bounded-work', marker='`ShapeSync::forget`', clause='Forget remembered shapes so the next supported apply can resend them.')
add([257], section='keyboard-list-layout-and-text-rules', marker='| Cursor shape |', clause='Reset shapes previously sent by the process.', doc='authoring', table_cell=1)
add([259], section='list-selection-and-type-ahead', marker='A prefix resets after', clause='Type-ahead uses a rolling prefix, resetting after a pause longer than its timeout and cycling on repeated single letters.')
add([261], section='list-selection-and-type-ahead', marker='The default timeout is one second', clause='The default type-ahead timeout is one second.')
add([263], section='list-selection-and-type-ahead', marker='Type-ahead accepts bare alphanumeric', clause='Accept bare alphanumeric keys; leave punctuation and modified keys available to the application.')
add([267], section='list-selection-and-type-ahead', marker='Type-ahead starts at the supplied row', clause='Search eligible labels from the supplied row with wrap; repeated one-letter prefixes start after the current row.')
add([268, 269], section='layout-and-bounded-work', marker='`ColumnSpec` constructors', clause='ColumnSpec constructors normalize min <= ideal <= max, and private fields preserve those bounds for the solver.')
add([270, 271], section='keyboard-list-layout-and-text-rules', marker='| Column layout |', clause='Keep hidden columns distinct from visible zero-width columns; preserve the documented drop, shrink, grow, and flex ordering.', doc='authoring', table_cell=1)
add([272], section='layout-and-bounded-work', marker='`solve_columns_first`', clause='The first-column helper avoids allocation through four specs and uses the full untruncated solver for larger slices.')
add([273], section='layout-and-bounded-work', marker='`solve_columns_head`', clause='The head helper avoids allocation through four specs, reports zero beyond the slice, and uses the full solver for larger inputs.')
add([274], section='keyboard-list-layout-and-text-rules', marker='| Composition and measurement |', clause='Composition planning resolves geometry only; measure each child once and use bounded, pure callbacks that do not handle input or cause domain effects.', doc='authoring', table_cell=1)
add([278, 279], section='layout-and-bounded-work', marker='When minima exceed', clause='Compress shrinks children in source order and clips final overflow; Clip preserves child minima and clips visible bounds.')
add([286], section='layout-and-bounded-work', marker='Intrinsic composition size', clause='Intrinsic size includes child sizes, padding, and gaps; alignment and flex consume only parent-supplied space.')
add([287, 288], section='keyboard-list-layout-and-text-rules', marker='| Composition and measurement |', clause='Measure callbacks receive bounded loose constraints and must not handle input or cause domain effects.', doc='authoring', table_cell=1)
add([289], section='layout-and-bounded-work', marker='Wrapping preserves source order', clause='Wrapping preserves source order, starts a new line before preferred width exceeds remaining content, resolves flex per line, and uses the tallest child height.')
add([290], section='layout-and-bounded-work', marker='Use basis points to retain', clause='Use precise basis points across resize while retaining coarse public percent as compatibility value.')
add([291], section='layout-and-bounded-work', marker='A first-pane extent', clause='Measure exact first-pane extent along the split axis, excluding the gap, before applying configured minima.')
add([293], section='layout-and-bounded-work', marker='A fixed first-share request', clause='Round a fixed first-share request to nearest cell and clamp it to pane bounds and the second pane minimum.')
add([294], section='layout-and-bounded-work', marker='A fixed first-share request', clause='Floor a fixed second-share request and clamp it to pane bounds and the first pane minimum.')
add([295], section='layout-and-bounded-work', marker='Required-extent methods use', clause='Required-extent methods call the matching resolver so measurement and drawing share rounding and pane minima.')
add([296, 297], section='layout-and-bounded-work', marker='Required-extent methods use', clause='Fixed first- and second-share required extents call their matching share layout resolver.')
add([298, 299], section='layout-and-bounded-work', marker='Basis-point drag and nudge', clause='Drag and nudge retain the precise ratio in basis points for later resizes; nudge moves the first pane by the requested cell delta.')

# Text editing boundaries and undo/redo lifecycle.
add([397], section='text-offsets-and-undo', marker='Selecting a line excludes', clause='Line selection excludes the newline.')
add([398], section='text-offsets-and-undo', marker='Insertion keeps', clause='Insert without consuming selection; clamp the byte offset and floor it to a grapheme boundary, then normalize cursor/anchor directions.')
add([399], section='text-offsets-and-undo', marker='Removal uses', clause='Remove a clamped half-open range; no-op empty/reversed ranges; preserve positions through the expanded-start/end rules and normalize grapheme boundaries.')
add([400], section='text-offsets-and-undo', marker='`TextBuffer::pos_of`', clause='Clamp past-end offsets; map a byte inside a UTF-8 scalar to its start; preserve valid boundaries even inside a grapheme.')
add([402], section='text-offsets-and-undo', marker='Record an undo snapshot', clause='Undo is snapshot-based; same-kind typing/deleting runs fold, and seal ends a run.')
add([404, 405, 406], section='text-offsets-and-undo', marker='Record an undo snapshot', clause='EditKind controls folding: consecutive typing or deleting records fold; Other breaks the run.')
add([407], section='text-offsets-and-undo', marker='An `Other` edit', clause='Other edits never fold and break the current run.')
add([408, 409], section='text-offsets-and-undo', marker='The stack keeps at most 200', clause='The bounded undo stack retains at most 200 steps.')
add([410], section='text-offsets-and-undo', marker='Record an undo snapshot before', clause='Record the snapshot before applying the edit; recording clears redo; consecutive Typing or Deleting records fold into the active step.')
add([411], section='text-offsets-and-undo', marker='An `Other` edit', clause='Seal ends the current run so the next record begins a new step.')
add([415], section='text-offsets-and-undo', marker='Undo restores', clause='Undo restores the prior snapshot and puts the current state on redo.')
add([416], section='text-offsets-and-undo', marker='Undo restores', clause='Redo restores the undone state and puts the current state on undo.')

# Confirm only the explicit rows above carry required/open/retirement semantics.
# Remaining rows were read as individual API labels, values, signatures, or history.

def text_digest(lines):
    packed = json.dumps(lines, ensure_ascii=False, separators=(',', ':')).encode()
    return hashlib.sha256(packed).hexdigest()

def read_git_bytes(commit, path):
    return subprocess.check_output(['git', 'show', f'{commit}:{path}'])

def module_name(path):
    m = re.match(r'crates/([^/]+)/src/(.*)\.rs$', path)
    if not m:
        return path
    crate = m.group(1).replace('-', '_')
    sub = m.group(2).replace('/', '::')
    if sub == 'lib':
        return crate
    return f'{crate}::{sub}'

def braces_open(lines, begin, end):
    depth = 0
    for raw in lines[begin:end]:
        line = raw.split('//', 1)[0]
        line = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
        depth += line.count('{') - line.count('}')
    return depth > 0

def active_container(lines, end_index, kind):
    if kind == 'method':
        candidates = [
            (i, re.match(r'^\s*impl(?:\s*<[^>]*>)?\s+(?:.*\s+for\s+)?([A-Za-z_]\w*)\b', line))
            for i, line in enumerate(lines[:end_index])
        ]
    else:
        pattern = r'^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_]\w*)\b' if kind == 'variant' else r'^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_]\w*)\b'
        candidates = [(i, re.match(pattern, line)) for i, line in enumerate(lines[:end_index])]
    for i, match in reversed(candidates):
        if match and braces_open(lines, i, end_index):
            return match.group(1)
    return None

def active_module(lines, end_index):
    for i in range(end_index - 1, -1, -1):
        m = re.match(r'^\s*pub\s+mod\s+([A-Za-z_]\w*)\b', lines[i])
        if m and braces_open(lines, i, end_index):
            return m.group(1)
    return None

def attached_item(path, text, source_lines, end_line):
    if any(line.lstrip().startswith('//!') for line in text):
        name = module_name(path)
        public_line = None
        for line in source_lines[end_line: end_line + 80]:
            clean = line.strip()
            if re.match(r'pub\s+(?:mod|trait|struct|enum|type|const|fn)\b', clean):
                public_line = clean
                break
        return {'kind': 'module_documentation', 'name': name,
                'signature': 'module-level Rustdoc', 'anchor_declaration': public_line}
    j = end_line
    while j < len(source_lines):
        stripped = source_lines[j].strip()
        if not stripped or stripped.startswith(('///', '//!', '//', '#[', '/*', '*')):
            j += 1
            continue
        break
    sig = source_lines[j].strip() if j < len(source_lines) else ''
    name = None
    m = re.search(r'\b(?:async\s+)?fn\s+([A-Za-z_]\w*)', sig)
    if m:
        base = m.group(1)
        parent = active_container(source_lines, j, 'method')
        name = f'{parent}::{base}' if parent else base
    else:
        m = re.search(r'\b(?:pub\s+)?(?:struct|enum|trait|type|mod|const|static|union)\s+([A-Za-z_]\w*)', sig)
        if m:
            name = m.group(1)
        else:
            m = re.match(r'([A-Z][A-Za-z0-9_]*)\s*(?:\{|\(|,|=)', sig)
            if m:
                parent = active_container(source_lines, j, 'variant')
                name = f'{parent}::{m.group(1)}' if parent else m.group(1)
            else:
                m = re.match(r'(?:pub\s+)?([a-z_][A-Za-z0-9_]*)\s*:', sig)
                if m:
                    parent = active_container(source_lines, j, 'field')
                    name = f'{parent}::{m.group(1)}' if parent else m.group(1)
    ident = module_name(path)
    nested = active_module(source_lines, j)
    if nested:
        ident = f'{ident}::{nested}'
    full_name = f'{ident}::{name}' if name else f'{ident}::<item at source line {j + 1}>'
    return {'kind': 'rust_item', 'name': full_name, 'signature': sig, 'source_line': j + 1}

def document_for(spec):
    if spec['doc'] == 'foundation':
        return GUIDE, FOUNDATION
    if spec['doc'] == 'authoring':
        return AUTH, AUTH_PATH
    if spec['doc'] == 'api_rules':
        return PLAN / 'API-RULES.md', API_RULES
    return None, None

def excerpt_for(spec):
    doc_path, rel_path = document_for(spec)
    if doc_path is None:
        return None
    content = doc_path.read_text()
    marker = spec['marker']
    markers = spec.get('support_markers', [marker])
    found = []
    for supported_marker in markers:
        supported_lines = [line.strip() for line in content.splitlines() if supported_marker in line]
        if not supported_lines:
            raise ValueError(f"no supporting excerpt for marker {supported_marker!r} in {doc_path}")
        found.append(supported_lines[0])
    if not found:
        raise ValueError(f"no supporting excerpt for marker {marker!r} in {doc_path}")
    if len(found) > 1:
        return '\n'.join(found)
    line = found[0]
    if spec.get('table_cell') is not None:
        cells = [x.strip() for x in line.split('|')]
        excerpt = cells[spec['table_cell'] + 1]
        if not excerpt:
            raise ValueError(f'empty table cell for {marker}')
    elif line.startswith('- '):
        excerpt = line[2:]
    else:
        excerpt = line
    if spec.get('table_cell') is None and marker not in excerpt:
        raise ValueError(f"support excerpt lost marker {marker!r}: {excerpt!r}")
    return excerpt

def clean_comment(text):
    vals = []
    for line in text:
        line = line.strip()
        line = re.sub(r'^/{2,3}!?', '', line).strip()
        line = re.sub(r'^\*+/?', '', line).strip()
        if line:
            vals.append(line)
    joined = re.sub(r'\s+', ' ', ' '.join(vals)).strip()
    return joined

def short_quote(text, limit=96):
    if len(text) <= limit:
        return text
    cut = text[:limit - 1].rsplit(' ', 1)[0]
    return cut + '…'

records = []
for source_map_record, raw in cohort:
    identity = cross_by_record[source_map_record]
    source_bytes = read_git_bytes(PRODUCT, raw['path'])
    source_lines = source_bytes.decode('utf-8').splitlines()
    clean_bytes = read_git_bytes(CLEANUP, raw['path'])
    clean_lines = clean_bytes.decode('utf-8').splitlines()
    item = attached_item(raw['path'], raw['text'], source_lines, identity['line_range']['end'])
    status = {
        'status': 'present_after_accepted_cleanup',
        'cleanup_commit': CLEANUP,
        'path': raw['path'],
        'source_sha256': hashlib.sha256(source_bytes).hexdigest(),
        'cleaned_file_sha256': hashlib.sha256(clean_bytes).hexdigest(),
    }
    if item['kind'] == 'module_documentation':
        if not item['anchor_declaration'] or item['anchor_declaration'] not in clean_bytes.decode('utf-8'):
            raise ValueError(f'module declaration missing after cleanup: {source_map_record} {item}')
        status['check'] = 'module path remains and its first public declaration remains'
        status['matched_declaration'] = item['anchor_declaration']
    else:
        if item['signature'] not in [line.strip() for line in clean_lines]:
            raise ValueError(f'attached item header missing after cleanup: {source_map_record} {item}')
        status['check'] = 'exact attached item header remains in cleaned source'
        status['matched_declaration'] = item['signature']
        status['match_count'] = sum(1 for line in clean_lines if line.strip() == item['signature'])
    spec = specs.get(source_map_record)
    if spec:
        excerpt = excerpt_for(spec)
        if spec['doc'] == 'foundation':
            dest = FOUNDATION
        elif spec['doc'] == 'authoring':
            dest = AUTH_PATH
        elif spec['doc'] == 'api_rules':
            dest = API_RULES
        else:
            dest = None
        destination = f"{dest}#{spec['section']}" if dest else None
        decision_text = spec.get('clause')
        if spec['disposition'] == 'source_wording_conflict':
            semantic = f"Open source wording decision: {spec['reason']}"
        elif spec['disposition'] == 'source_observation':
            semantic = f"Source observation only: {spec['reason']}"
        elif spec['disposition'] == 'planned_api_retirement':
            semantic = f"Planned T20 retirement for {item['name']}: {spec['reason']}"
        else:
            semantic = f"Preserve this caller rule for {item['name']}: {decision_text}"
        reason = spec['reason']
        decision_id = spec.get('decision_id')
        disposition = spec['disposition']
    else:
        quote = short_quote(clean_comment(raw['text']))
        if any(line.lstrip().startswith('//!') for line in raw['text']) and re.search(r'ported from|ported from PR|commit [0-9a-f]{7,}', clean_comment(raw['text']), re.I):
            disposition = 'historical_or_internal_description'
            reason = f"This module note for {item['name']} records port or repository history: “{quote}”. It adds no caller precondition or runtime guarantee. Retain the exact span and text as provenance."
        else:
            disposition = 'descriptive_api_reference'
            reason = f"The comment for {item['name']} says: “{quote}”. It names the API element, value, or direct result; it states no separate caller precondition, lifecycle/failure, identity/selection, callback/borrow, or bounded-work rule. Keep it as a descriptive T01/T35 API reference."
        semantic = f"{disposition}: {reason}"
        decision_text = None
        destination = None
        excerpt = None
        decision_id = None
    records.append({
        'id': identity['id'],
        'source_map_record': identity['source_map_record'],
        'path': identity['path'],
        'line_range': identity['line_range'],
        'source_sha256': identity['source_sha256'],
        'text_sha256': identity['text_sha256'],
        'source_text': raw['text'],
        'attached_item': item,
        'cleanup_target_status': status,
        'disposition': disposition,
        'semantic_decision': semantic,
        'required_clause': decision_text,
        'destination': destination,
        'supporting_excerpt': excerpt,
        'specific_reason': reason,
        'related_decision': decision_id,
    })

if len(cohort) != 322 or len(records) != 322:
    raise ValueError(f'wrong cohort size: {len(records)}')
if set(specs) - {i for i, _ in cohort}:
    raise ValueError(f'specs outside cohort: {set(specs) - {i for i, _ in cohort}}')
if len({r['id'] for r in records}) != 322:
    raise ValueError('duplicate ids')
if len({r['source_map_record'] for r in records}) != 322:
    raise ValueError('duplicate source map record')

# Keep original source identities and hashes unchanged; independently verify their bytes.
for record in records:
    i = record['source_map_record']
    raw = raw_by_record[i]
    old = cross_by_record[i]
    expected_keys = ('id', 'source_map_record', 'path', 'line_range', 'source_sha256', 'text_sha256')
    if any(record[k] != old[k] for k in expected_keys):
        raise ValueError(f'identity changed for {i}')
    if record['path'] != raw['path'] or record['line_range']['start'] != raw['start_line'] or record['line_range']['end'] != raw['end_line']:
        raise ValueError(f'raw span mismatch for {i}')
    if text_digest(raw['text']) != record['text_sha256']:
        raise ValueError(f'text hash mismatch for {i}')
    if hashlib.sha256(read_git_bytes(PRODUCT, record['path'])).hexdigest() != record['source_sha256']:
        raise ValueError(f'source hash mismatch for {i}')

api_retirement = 'T20 removes core\'s global protocol and delivery mirrors.'
for r in records:
    if r['disposition'] == 'planned_api_retirement' and api_retirement not in r['supporting_excerpt']:
        raise ValueError(f'missing exact retirement decision for {r["source_map_record"]}')

out = {
    'schema': 't37.api-values-cohort-decisions.v1',
    'task_id': 'T37',
    'decision_id': 'T37-API-PROSE-01',
    'review_state': 'PENDING_INDEPENDENT_HIGH_REVIEW',
    'work_input_commit': CONTRACTS,
    'fixed_input_commit': INPUT,
    'product_source_commit': PRODUCT,
    'raw_inventory': {
        'path': 'docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json',
        'commit': RAW_COMMIT,
        'sha256': crosswalk['source_inventory']['sha256'],
        'record_count': 1122,
    },
    'identity_source': {
        'path': 'docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json',
        'commit': INPUT,
        'schema': crosswalk['schema'],
        'identity_fields_preserved': ['id', 'source_map_record', 'path', 'line_range', 'source_sha256', 'text_sha256'],
    },
    'cleanup_source': {
        'path': 'docs/implementation/pr38-final/results/t37-foundations/source-removal.json',
        'commit': RAW_COMMIT,
        'accepted_source_commit': CLEANUP,
        'source_comment_categorization_commit': '2fee2c277d1213fc5fad5d81eb77479600182b69',
    },
    'cohort': {
        'paths': ['crates/termrock-core/', 'crates/termrock-layout/', 'crates/termrock-text/'],
        'counts': {'crates/termrock-core/': 268, 'crates/termrock-layout/': 32, 'crates/termrock-text/': 22},
        'record_count': 322,
        'ordering': 'Sorted by source_map_record for readability only; id is the stable identity.',
        'semantic_method': 'Read each complete raw group with immutable surrounding source and accepted cleaned source; decisions are group-specific and do not reuse lexical dispositions as semantic truth.',
        'selected_behavior_cases': 0,
        'behavior_status': 'NOT VERIFIED',
    },
    'testing_retirement_crosscheck': {
        'source_removal_record': 'docs/implementation/pr38-final/results/t37-foundations/source-removal.json',
        'retired_testing_api_group_ids': removal['retired_testing_api_group_ids'],
        'assigned_cohort_retired_test_only_groups': [],
        'core_intent_queue_probes': 'No assigned group attaches to the retired test-only probes; production claim APIs at intent.rs groups 13-21 remain documented.',
        'theme_fingerprint': 'The test-only Theme::fingerprint retirement is outside these three crate paths.',
        'secret_fingerprint': {
            'path': 'crates/termrock-text/src/secret.rs',
            'status': 'public production method remains after cleanup; no Rustdoc group in the assigned cohort attaches to it',
            'cleanup_commit': CLEANUP,
        },
        'runtime_testing_projection_style_stub_helpers': 'Outside the assigned API group cohort and retained as their explicit T37 testing-surface retirements.',
    },
    'explicit_review_decisions': [
        {'id': 'T37-NAV-STATE-01', 'reviewer': '/root/t00_independent_review', 'result': 'thread-local defaults and install/current semantics preserved'},
        {'id': 'T37-KEY-OBSERVE-01', 'reviewer': '/root/t00_independent_review', 'result': 'exact Super/Alt observation predicate preserved; T20/T19 ownership retained'},
        {'id': 'T37-DIAGNOSTICS-01', 'reviewer': '/root/t00_independent_review', 'result': 'capacity-bounded items and cumulative saturating dropped counter; no defect established'},
        {'id': 'T37-SELECTION-CONTRACT-01', 'reviewer': '/root/t00_independent_review', 'result': 'saved-base reconstruction and operation-specific disabled-row semantics'},
        {'id': 'T37-CURSOR-NEW-01', 'reviewer': '/root/t00_independent_review', 'result': 'constructor stores input index and defaults axis/wrap/skip policy; no defect established'},
    ],
    'records': records,
}
path = OUTDIR / 'decisions.json'
path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + '\n')
print(f'wrote {path} ({len(records)} records, {len(specs)} explicit maintained/open/retired decisions)')
print('dispositions:', json.dumps({k: sum(r['disposition'] == k for r in records) for k in sorted({r['disposition'] for r in records})}))
print('decisions requiring support:', sum(bool(r['destination']) for r in records))
print('direct declarations:', sum(r['attached_item']['kind'] == 'rust_item' for r in records))
print('module groups:', sum(r['attached_item']['kind'] == 'module_documentation' for r in records))
