#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path
from typing import Any


WORK_INPUT = "e164e7b5b17e9c2d78bf30cfbbfe6bdb36e7b7ab"
SOURCE_INPUT = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
PRODUCT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
FOUNDATION_OUTPUT = "2fee2c277d1213fc5fad5d81eb77479600182b69"
THEME_DATA_DECISION = "T37-THEME-DATA-01"
THEME_DATA_REVIEWER = "/root/t00_independent_review"
FOOTER_PIN_DECISION = "T37-FOOTER-PIN-CONTRACT-01"
FOOTER_PIN_REVIEWER = "/root/t00_independent_review"
FOOTER_PIN_INDICES = {762, 763, 764, 771, 772}
VISIT_KEY_DECISION = "T37-VISIT-KEY-01"
VISIT_KEY_INDICES = {803, 813}
INTERNAL_INDICES = {423, 424, 425, 426, 376, 740, 985}
SOURCE_MAP_SHA = "6ead70836c20e3ad1c2dc15f0de44daf35d74c6507e1bbfd086de5fa6e85b619"
SOURCE_DISPOSITION_SHA = "8e3923ebb4f6d5974a2e632ca32e5406718a5985a8f26a17a75fcbd9a3f4e9df"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def normalized(comment_lines: list[str]) -> str:
    return " ".join(re.sub(r"^\s*//!?\s?", "", line).strip() for line in comment_lines).strip()


def source_bytes(repo: Path, path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{SOURCE_INPUT}:{path}"], cwd=repo)


def declaration(path: str, source: list[str], start: int, end: int) -> tuple[str, str, str | None, str, int | None]:
    if start == 1 and source and source[0].lstrip().startswith("//!"):
        return f"module documentation for `{path}`", "", None, "module", None
    following = ""
    declaration_line = None
    for number, line in enumerate(source[end:], end + 1):
        candidate = line.strip()
        if not candidate or candidate.startswith(("///", "//!", "#[")):
            continue
        following = candidate
        declaration_line = number
        break
    direct = re.match(
        r"^(?:(?:pub(?:\([^)]*\))?)\s+)?(?:(?:async|unsafe|const|extern(?:\s+\"[^\"]+\")?)\s+)*(fn|struct|enum|trait|mod|const|static|type)\s+([A-Za-z_][A-Za-z0-9_]*)",
        following,
    )
    if direct:
        kind, name = direct.groups()
        return f"{kind} `{name}` (next declaration: `{following[:120]}`)", name, following, kind, declaration_line
    member = re.match(r"^(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::|,|=|\{|\()", following)
    if member:
        name = member.group(1)
        return f"member `{name}` (next declaration line: `{following[:120]}`)", name, following, "member", declaration_line
    if following:
        return f"next Rust item line `{following[:120]}`", "", following, "unknown", declaration_line
    return f"source group at `{path}:{start}-{end}`", "", None, "unknown", None


def rust_tokens(lines: list[str]) -> list[tuple[str, int]]:
    text = "\n".join(lines)
    tokens: list[tuple[str, int]] = []
    i = 0
    line = 1
    while i < len(text):
        char = text[i]
        if char == "\n":
            line += 1
            i += 1
            continue
        if char.isspace():
            i += 1
            continue
        if text.startswith("//", i):
            end = text.find("\n", i)
            i = len(text) if end < 0 else end
            continue
        if text.startswith("/*", i):
            depth = 1
            i += 2
            while i < len(text) and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    if text[i] == "\n":
                        line += 1
                    i += 1
            continue
        raw_start = re.match(r"(?:br|r)(#+)?\"", text[i:])
        if raw_start:
            hashes = raw_start.group(1) or ""
            opening = raw_start.end()
            closing = '"' + hashes
            end = text.find(closing, i + opening)
            segment_end = len(text) if end < 0 else end + len(closing)
            line += text.count("\n", i, segment_end)
            tokens.append(("LITERAL", line))
            i = segment_end
            continue
        string_start = re.match(r"(?:b|c)?\"", text[i:])
        if string_start:
            i += string_start.end()
            while i < len(text):
                if text[i] == "\\":
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    if text[i] == "\n":
                        line += 1
                    i += 1
            tokens.append(("LITERAL", line))
            continue
        if char == "'":
            close = i + 1
            if close < len(text) and text[close] == "\\":
                close += 2
                if close < len(text) and text[close] == "'":
                    i = close + 1
                    tokens.append(("LITERAL", line))
                    continue
            elif close + 1 < len(text) and text[close + 1] == "'":
                i = close + 2
                tokens.append(("LITERAL", line))
                continue
            tokens.append(("'", line))
            i += 1
            continue
        ident = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i:])
        if ident:
            token = ident.group(0)
            tokens.append((token, line))
            i += len(token)
            continue
        tokens.append((char, line))
        i += 1
    return tokens


def container_name(header: list[str]) -> tuple[str, str] | None:
    values = list(header)
    while values and values[0] == "#":
        values = values[1:]
        if values and values[0] == "[":
            depth = 0
            while values:
                token = values.pop(0)
                if token == "[":
                    depth += 1
                elif token == "]":
                    depth -= 1
                    if depth == 0:
                        break
    if values and values[0] == "pub":
        values.pop(0)
        if values and values[0] == "(":
            depth = 0
            while values:
                token = values.pop(0)
                if token == "(":
                    depth += 1
                elif token == ")":
                    depth -= 1
                    if depth == 0:
                        break
    while values and values[0] in ("unsafe", "async", "const", "default", "auto"):
        values.pop(0)
    if not values or values[0] not in ("mod", "struct", "enum", "trait", "impl", "union"):
        return None
    kind = values.pop(0)
    if kind != "impl":
        name = next((token for token in values if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", token)), "")
        return (kind, name) if name else None
    while values and values[0] == "<":
        depth = 0
        while values:
            token = values.pop(0)
            if token == "<":
                depth += 1
            elif token == ">":
                depth -= 1
                if depth == 0:
                    break
    if "where" in values:
        values = values[:values.index("where")]
    if "for" in values:
        at = values.index("for")
        return ("impl", "impl " + " ".join(values[:at]) + " for " + " ".join(values[at + 1:]))
    return ("impl", "impl " + " ".join(values)) if values else ("impl", "impl")


def lexical_scopes(lines: list[str]) -> dict[int, tuple[str, ...]]:
    tokens = rust_tokens(lines)
    frames: list[tuple[str, str, int] | None] = []
    headers: list[list[str]] = [[]]
    scopes: dict[int, tuple[str, ...]] = {}
    for token, line in tokens:
        scopes[line] = tuple(f"{kind}:{name}" for kind, name, _ in frames if kind)
        if token == "{":
            item = container_name(headers[-1])
            frames.append((item[0], item[1], line) if item else ("", "", line))
            headers.append([])
        elif token == "}":
            if frames:
                frames.pop()
            if len(headers) > 1:
                headers.pop()
            headers[-1] = []
        elif token == ";":
            headers[-1] = []
        else:
            headers[-1].append(token)
    return scopes


SUPPORT = "docs/contracts/foundation-support.md"


def spec(anchor: str, excerpt: str, clause: str) -> dict[str, str]:
    return {
        "semantic_disposition": "required_caller_contract",
        "destination": f"{SUPPORT}#{anchor}",
        "supporting_excerpt": excerpt,
        "required_clause": clause,
    }


RULES: dict[int, list[dict[str, str]]] = {}


def add(indices: list[int], anchor: str, excerpt: str, clause: str) -> None:
    for index in indices:
        RULES.setdefault(index, []).append(spec(anchor, excerpt, clause))


add([418], "scroll-and-keyboard-sessions", "`FollowCursor::OnMove` reveals after the cursor changes.", "OnMove follows a changed cursor.")
add([419], "scroll-and-keyboard-sessions", "`Always` keeps the cursor in view on each pass.", "Always rechecks visibility on every pass.")
add([421], "scroll-and-keyboard-sessions", "`TrackClick::Jump` centers the thumb at the pointer.", "Jump centers the scrollbar thumb on the pointer.")
add([422], "scroll-and-keyboard-sessions", "`TrackClick::Page` moves one viewport toward the pointer and starts a paging drag.", "Page clicks page one viewport and keep paging during the drag.")
add([431], "scroll-and-keyboard-sessions", "A pointer-driven reveal centers the row with no context.", "Pointer reveal is exact and uses no context rows.")
add([432], "scroll-and-keyboard-sessions", "`reveal_span` keeps the requested context when the viewport permits it. A span at least as long as the viewport starts at the top.", "Span reveal preserves available context and top-aligns a span at least as tall as the viewport.")
add([433], "scroll-and-keyboard-sessions", "Repeating the same cursor in the same viewport is a no-op, so repaint does not undo wheel movement.", "OnMove deduplicates the cursor/viewport pair so repaint does not undo wheel movement.")
add([434], "scroll-and-keyboard-sessions", "`TrackClick::Jump` centers the thumb at the pointer. `TrackClick::Page` moves one viewport toward the pointer and starts a paging drag.", "The selected track-click mode controls both the press and the continued paging drag.")

add([1114, 1115, 1116, 1117, 1118], "scroll-and-keyboard-sessions", "Negotiation is opt-in. A session that does not call `enter` writes no protocol bytes. `enter` pushes at most one flag sequence and flushes; `leave` pops only a sequence this state pushed, once; `resume` pushes once after suspension.", "Keyboard-protocol session entry is opt-in and push/pop/resume operations are bounded to the current session state.")
add([1116], "scroll-and-keyboard-sessions", "In `Auto` mode, the caller supplies `detect` from its terminal capability probe", "Auto mode requires a caller-provided capability result.")
add([1119, 1120, 1121], "scroll-and-keyboard-sessions", "`state`, `is_pushed`, and `is_enabled` only read state and write no bytes.", "State accessors do not write terminal protocol bytes.")

add([300, 314, 315, 325, 326, 327, 328], "theme-presentation-policy", "A `Theme` carries its explicit shared-Manager presentation policy. Cloning or downgrading the theme keeps that policy. A builder made from the theme inherits it until the caller changes it.", "Shared-Manager presentation policy survives clone/downgrade and is inherited by a builder unless changed.")
add([315, 327, 328], "theme-presentation-policy", "`Paper` selects Paper semantic ink at Mono and TrueColor; its saved Manager layout applies only to an alert-free Saved frame without the detail drawer.", "Paper policy is limited to the stated tiers and Saved-frame conditions.")
add([316], "theme-presentation-policy", "Family profiles replace only channels that still match the family recipe captured when the first profile was defined.", "Family profiles preserve caller changes to channels that no longer match the captured family recipe.")
add([317], "theme-presentation-policy", "Variant profiles apply after family defaults and before global, scope, and instance overrides.", "Variant profile precedence is between family defaults and ordinary overrides.")
add([318], "theme-presentation-policy", "A surface profile selects by rendering surface; terminal dimensions and application identity do not select it.", "Surface profiles use rendering surface, not dimensions or app identity.")
add([307], "modal-backdrop-and-color-downgrade", "The source's named profile corrections follow the general color cube. The matching TablePro custom role uses the same RGB.", "The recorded apply_256 exception follows the cube and keeps the matching TablePro RGB; it is not a general conversion guarantee.")
add([308], "modal-backdrop-and-color-downgrade", "The pinned raw RGB example preserves reverse for the top brand chip. The same green elsewhere maps to black on white without reverse.", "The pinned raw-RGB case distinguishes the top brand chip from the same green elsewhere.")
add([309], "modal-backdrop-and-color-downgrade", "In the 16-color example, white on dark gray maps to black to retain the frozen label contrast.", "The cited 16-color pair is a frozen example, not a general RGB guarantee.")
add([319], "modal-backdrop-and-color-downgrade", "Baseline modal dim keeps glyphs, makes accent and secondary ink faint, and keeps ghost gray when a color already fails contrast.", "Modal dim preserves glyphs and keeps ghost gray when contrast already fails.")
add([320], "modal-backdrop-and-color-downgrade", "At ANSI16 and Mono, covered cells use muted dim ink because those planes do not carry the per-source ghost ladder.", "Low-color modal dim uses muted ink when the per-source ghost ladder is unavailable.")
add([321], "modal-backdrop-and-color-downgrade", "`Theme::backdrop_cell` also returns whether its source cell was blank. At ANSI16 and Mono, its covered cells use the muted dim ink, matching `backdrop_style`.", "The concrete backdrop helper reports source blankness and matches the low-color style mapping.")
add([322], "modal-backdrop-and-color-downgrade", "Preserve the glyph, replace modifiers, and set `DIM` only on nonblank dim cells.", "Backdrop mapping preserves symbols, replaces modifiers, and dims only nonblank cells.")
add([329, 330, 331], "theme-presentation-policy", "`RecipeEdit` records only the intent that the caller expressed. It does not infer ownership by comparing values with a built-in recipe. Keep `Inherit`, `Set`, and `Clear` distinct when declaring glyph or size channels.", "Recipe slots preserve caller intent, including explicit clear and inherit; ownership is not inferred from value equality.")
add([334], "theme-presentation-policy", "`bind_defaults_with_ownership` returns the same resolved paint as `bind_defaults` and a separate ownership patch. The patch includes eligible profiles and authored Mono rules, then global and scoped overrides and the instance patch in resolver order. Built-in Mono fallback rules do not claim ownership.", "The ownership patch follows resolver precedence without treating built-in Mono fallback as explicit ownership.")
add([335, 343], "theme-presentation-policy", "`Role` and `ColorTokens` are non-exhaustive. Downstream matches must include a fallback arm for variants added later. Do not rely on a complete `ColorTokens` struct literal.", "Downstream code must allow future Role variants and must not construct a complete ColorTokens literal.")
add([351], "focus-selection-and-gutter-policy", "The theme helpers expose shared policies. Source comments say existing component families still paint through recipe parts until they adopt the helpers.", "Theme helpers are not a claim that all component families use them.")
add([352, 353, 354, 355, 371, 373, 374], "focus-selection-and-gutter-policy", "Focused items use accent ink. An unfocused item uses accent only when its `InactiveSelection` policy is `Accent`; otherwise it uses primary text.", "Focused and inactive selected-item ink follows InactiveSelection policy.")
add([352, 364, 365, 366, 372, 373, 374], "focus-selection-and-gutter-policy", "Under a modal or popup, those marks use the neutral policy unless the explicit backdrop policy selects accent.", "Modal/backdrop selection changes only under the explicit BackdropSelection policy.")
add([356, 357, 358, 359, 360], "focus-selection-and-gutter-policy", "Button bars need focus and focus-visible, and remain visible when disabled. List bars need focus and enabled state at every tier. Choice, Tree, and Steps draw on focus, including disabled rows, at TrueColor, ANSI256, and ANSI16; Mono blanks a disabled row. Tier-sensitive policies read the theme's own capability level.", "The shared focus-bar policies retain family- and tier-specific rules.")
add([361, 362, 363, 370], "focus-selection-and-gutter-policy", "`GutterCell::Inherit` stamps only the focus bar foreground and leaves the rest of the cell to the container. `GutterCell::Fill` stamps an explicit blank in the host background.", "Gutter-cell policy distinguishes inherited container paint from an explicit host-background fill.")
add([367], "focus-selection-and-gutter-policy", "`gutter_symbol` uses `RowBlanked`: it returns the phosphor bar only while the widget is focused and enabled, and otherwise reserves a blank cell.", "The default gutter helper requires focus and an enabled widget.")
add([368, 369], "focus-selection-and-gutter-policy", "Use `focus_bar_symbol` with an explicit family policy when those defaults do not match the widget.", "Caller-selected focus-bar policy determines the symbol and gutter style.")
add([375, 377], "focus-selection-and-gutter-policy", "Choose ink against the background. Try the input, a brighter named-ANSI color, Danger and Success, primary text, then the more legible of primary or on-accent ink. If either color has no known RGB, including `Color::Reset`, the contrast ratio is unknown and the pair does not pass this check.", "Legible-mark fallback order is fixed; unknown RGB never passes the contrast check.")
add([378, 379, 380, 381], "descriptive-and-historical-source-notes", "The retained public theme arrays expose the declared values and order: `COLOR_LEVELS` is `[TrueColor, Ansi256, Ansi16, Mono]`; `FOCUS_BAR_POLICIES` follows `FocusBarPolicy::ALL` as `[RowBlanked, RowKept, FocusVisible]`; `GUTTER_CELLS` follows `GutterCell::ALL` as `[Inherit, Fill]`; and `BACKDROP_SELECTIONS` follows `BackdropSelection::ALL` as `[Neutral, Accent]`.", "Preserve current public array values and order as compatibility data; do not infer coverage or permanent exhaustiveness.")
add([311, 312, 313], "focus-selection-and-gutter-policy", "`CursorStyle::Bar` selects the theme focus gutter. `CursorStyle::Fill` highlights the active row and reserves a blank gutter cell. These are public policy choices; this note does not certify every widget's rendering.", "CursorStyle declares Bar and Fill policies; preserve their stated meanings without a widget-wide rendering claim.")

add([756, 751, 1054], "shell-layout-status-and-reports", "The default shell geometry is one header row, one gap row, one footer row, and no inset. Keep the strip rows while the area can hold them; the body uses the remaining rows and may be empty.", "Chrome splitting preserves strip rows when possible and uses the remainder for the body; defaults are fixed.")
add([762, 763], "shell-layout-status-and-reports", "The source comment says pinned hints survive errors, so it conflicts with the current assignment order.", "Preserve the pinned-survival promise and current normalization order as separate source facts until T19 resolves their precedence.")
add([765, 766], "shell-layout-status-and-reports", "`Hint::new` starts at `HINT_AUTO`.", "New hints default to HINT_AUTO unless the caller changes priority.")
add([772], "shell-layout-status-and-reports", "The first non-empty layer hint list wins; an empty list does not override earlier hints or fallback.", "The first non-empty layer list takes precedence; an empty list leaves the previous choice intact.")
add([772], "shell-layout-status-and-reports", "`compose` preserves that order, appends Tab-next once when enabled and not editing, and filters labels: it rejects recognized Shift-pointer labels and requires recorded delivery for recognized Cmd/Super and non-pointer Alt/Option prefixes.", "Tab-next is added once only outside edit mode, and compose applies the documented label filter.")
add([772], "shell-layout-status-and-reports", "The filter records label policy; it does not prove terminal support or emitted output.", "Hint filtering records label policy only; it does not prove terminal support or output.")
add([772], "shell-layout-status-and-reports", "If the status tone is `Role::Danger`, it then assigns `HINT_NORMAL` to every remaining hint, including pinned and custom-priority hints. This normalization does not remove or reorder hints; `HINT_AUTO` changes from 0 to 1.", "The source error wording is retained as a conflict while current code normalizes every returned priority.")
add([764, 771], "shell-layout-status-and-reports", "If the status tone is `Role::Danger`, it then assigns `HINT_NORMAL` to every remaining hint, including pinned and custom-priority hints. This normalization does not remove or reorder hints; `HINT_AUTO` changes from 0 to 1.", "The error path assigns a common priority and does not necessarily reduce each prior numeric value.")
add([781], "shell-layout-status-and-reports", "The first non-empty layer hint list wins; an empty list does not override earlier hints or fallback.", "Layer hint folding selects the first non-empty list and uses fallback only when no layer supplies one.")
add([790], "scroll-and-keyboard-sessions", "The shell observes modifier delivery and filters hint labels; the terminal session owns protocol push/pop.", "Shell modifier-delivery hints and terminal keyboard-mode protocol have separate owners.")
add([801], "shell-layout-status-and-reports", "The shell supplies routing vocabulary and composes its leaves through the runtime. The runtime owns focus, hit testing, pointer routing, layers, and the update loop.", "Shell/runtime ownership is split; the removed reachability harness is not part of the retained shell surface.")
add([805], "shell-routing-and-page-identity", "`PageKey::of` hashes the caller's value; include route and visit token when same-route re-entry must differ, but do not treat a 64-bit hash as a collision-free domain key.", "PageKey::of is hash-based; callers that need same-route visit distinction must include a visit token and still account for the open counter identity limit.")
add([803, 813], "shell-routing-and-page-identity", "Source comments promise that revisits get different keys and describe the counter as monotonic. The implementation advances one shared `u64` visit counter with `wrapping_add(1)`; `key()` uses zero as its absent-entry fallback. It therefore provides no non-reuse guarantee at counter exhaustion. T19-VISIT-KEY-01 remains open: define safe identity behavior at exhaustion before claiming unbounded distinct visits.", "Preserve the revisit-identity promise as an open requirement; do not claim non-reuse at counter exhaustion.")
add([989, 992], "shell-routing-and-page-identity", "`KeyBinding.priority` is an ordering weight for footer hints; lower values come first when a consumer uses it to order hints. The shell exposes this field but has no production consumer that sorts or renders it in the fixed source tree. This records the declared field meaning, not current presentation behavior. T01 must inventory its current owner.", "Preserve the declared lower-first ordering meaning for KeyBinding.priority without claiming current product sorting or rendering.")
add([1024, 1025, 1026], "shell-routing-and-page-identity", "Run the `priority` tier before page input. With a key-capturing component, give the page a chance before the `keys` tier. Otherwise, run the `keys` tier, then app chrome, then the page. The `keys` tier holds the shell's own bindings. Run `fallback` only after earlier handlers decline.", "Priority is before the page; Keys is the shell tier; Fallback runs after earlier handlers decline.")
add([899, 900], "shell-routing-and-page-identity", "A `Resolved::Save` result means save first, then carry out the pending leave; `Resolved::Discard` means abandon unsaved changes, then carry it out.", "Save before continuing the leave; abandon unsaved changes before continuing after Discard.")
add([1094], "shell-layout-status-and-reports", "`end_job` returns true only when it ends the message currently owned by that job.", "end_job reports and clears only a visible message owned by the requested job.")
add([806, 807, 808, 809, 810, 811, 812], "shell-routing-and-page-identity", "`Push` adds a page above the active top without dropping pages. `Replace` swaps and drops the old top. `Reset` replaces the active stack with a new root and drops every old entry. `Back` drops the top only when the stack has more than its root. `PopTo(d)` keeps `d + 1` entries and drops later entries when any remain. `Tab` switches stacks without dropping pages.", "The Nav operation determines stack mutation and the pages that must pass through the leave guard.")
add([809, 834], "shell-routing-and-page-identity", "`Reset` replaces the active stack with a new root and drops every old entry.", "Reset replaces the active stack, so dropped pages must be checked against their stakes.")
add([815], "shell-routing-and-page-identity", "`Router::with_tabs` requires at least one root and panics when the root list is empty.", "with_tabs has a nonempty-root precondition and panics on an empty root list.")
add([816], "shell-routing-and-page-identity", "The router keeps an active tab; its optional active-stack accessor returns no stack if that invariant fails, rather than panicking.", "The active-stack accessor is total and returns None when the active-tab invariant is broken.")
add([831], "shell-routing-and-page-identity", "A pop keeps the root; closing a tab returns no result for an invalid index or the last remaining tab, and a successful close returns its pages while preserving a tab.", "Back/pop retains the root entry.")
add([835], "shell-routing-and-page-identity", "`select` returns false for an out-of-range or already active tab.", "Tab selection reports false for invalid or no-op selection.")
add([836], "shell-routing-and-page-identity", "Opening a tab creates and activates its root stack.", "Opening selects the new root stack.")
add([837], "shell-routing-and-page-identity", "closing a tab returns no result for an invalid index or the last remaining tab, and a successful close returns its pages while preserving a tab.", "Closing rejects an invalid or final tab and returns removed page ownership while preserving a tab.")
add([838], "shell-routing-and-page-identity", "A move that drops no page needs no leave confirmation.", "A move with no dropped page does not require a leave guard.")
add([839], "shell-routing-and-page-identity", "Check dropped pages in top-first order and pass their stakes through the leave guard before applying a move.", "Check each dropped page in top-first order before applying the move.")
add([840], "shell-routing-and-page-identity", "Apply a confirmed move only after its leave dialog resolves; the result lists dropped pages top first.", "Apply a guarded move after dialog resolution and preserve top-first dropped-page reporting.")
add([842, 848], "shell-routing-and-page-identity", "Pass the number of extra rendered levels to click mapping; those levels shift the clicked index to its `PopTo` depth.", "Breadcrumb click mapping accounts for extra rendered levels.")
add([841, 843, 844, 845, 846, 847], "shell-routing-and-page-identity", "Crumb segments are built from the active stack, root first. Use their emitted child IDs when registering hits. Every segment except the final one links back.", "Breadcrumb labels follow stack order, use emitted child IDs, and expose only nonfinal links.")
add([849], "shell-routing-and-page-identity", "Every segment except the final one links back.", "Only nonfinal breadcrumb segments are links.")
add([850], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents. Use `on_show` for enter work.", "Page updates own queued intents and on_show owns enter work.")
add([852], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents.", "Page event kinds arrive as owner-scoped intents to update.")
add([853], "shell-layout-status-and-reports", "Draw into the shell body region.", "Page drawing uses the body region supplied by the shell.")
add([854], "shell-layout-status-and-reports", "Use the shell-resolved body bounds for layout-relative drawing and registration.", "Layout-relative drawing and hit registration use the shell-provided body bounds.")
add([857], "shell-routing-and-page-identity", "Focus upkeep compares page keys to identify page changes and focus a page's primary control when it appears.", "Page primary focus is coupled to page appearance and visit identity.")
add([858], "shell-layout-status-and-reports", "appends Tab-next once when enabled and not editing", "Tab-next is hidden while the text field owns typing.")
add([859], "shell-layout-status-and-reports", "Report `animating` only while the runtime should keep repainting.", "The animation flag controls continued runtime repainting.")
add([860], "shell-layout-status-and-reports", "Use `on_show` for enter work.", "on_show is the page-enter hook.")
add([861], "shell-routing-and-page-identity", "when no selection exists, the page interrupt hook may cancel work before the quit request.", "The interrupt hook may cancel page work when Ctrl+C has no selection to copy.")
add([855], "shell-routing-and-page-identity", "Crumb segments are built from the active stack, root first.", "A Page label names its segment in the root-first navigation trail.")
add([856], "shell-routing-and-page-identity", "Check dropped pages in top-first order and pass their stakes through the leave guard before applying a move.", "Page stakes are evaluated for each dropped page before navigation applies.")
add([862], "shell-routing-and-page-identity", "When a menu or hint represents a key, use `replay_key`. It maps newline to Enter, arrows to arrow keys, Escape to Esc, and other characters to an unmodified character key.", "Replayed menu/hint keys use the defined character-to-key mapping.")
add([863], "shell-layout-status-and-reports", "`rebuild_on_tick` runs the update closure and reports whether the view changed. Use that result to avoid repainting an unchanged view.", "Tick rebuild result reports view change and can avoid unchanged repaint.")
add([865], "shell-layout-status-and-reports", "The `page_leaf` adapter borrows the page for the returned leaf's use.", "The borrowed page must outlive the composition leaf use.")
add([867, 868], "shell-routing-and-page-identity", "A clean stake returns the requested leave without a dialog.", "A clean stake leaves without opening a confirmation dialog.")
add([869, 870], "shell-routing-and-page-identity", "Unsaved stakes record the count of changes at risk.", "The Unsaved stake carries the count of changes at risk.")
add([871, 872], "shell-routing-and-page-identity", "Running stakes name work that stopping abandons.", "The Running stake names the work that stopping abandons.")
add([874], "shell-routing-and-page-identity", "Running work outranks unsaved changes; unsaved counts add with saturation.", "Combining stakes prioritizes running work and saturates unsaved counts.")
add([876, 881], "shell-routing-and-page-identity", "Ctrl+C classifies as Copy when a selection exists and Quit otherwise", "A selection changes Ctrl+C from a quit request into a copy action.")
add([877, 884], "shell-routing-and-page-identity", "Ctrl+Q classifies as Quit when enabled, then the leave guard handles stakes.", "Enabled Ctrl+Q asks the leave guard to handle stakes.")
add([879], "shell-routing-and-page-identity", "Let a text field receive bare letters while it owns typing.", "A text field retains bare-letter input while it owns typing.")
add([880, 885, 893], "shell-routing-and-page-identity", "The configured letter classifies as Quit only when no text field owns typing and the page is clean", "A dirty page or active text field blocks the configured quit letter.")
add([882, 883, 887], "shell-routing-and-page-identity", "The defaults are Ctrl+C, Ctrl+Q, and `q`; Escape does not quit.", "Preserve the declared default quit chords and letter.")
add([886, 889], "shell-routing-and-page-identity", "Escape quitting is opt-in, only applies without text focus, and follows earlier handlers.", "Escape may request quit only when enabled, without text focus, and after earlier handlers decline.")
add([888], "shell-routing-and-page-identity", "callers can set another letter or disable it with `None`.", "The caller may change or remove the configured quit letter.")
add([890], "shell-routing-and-page-identity", "Ctrl+C classifies as Copy when a selection exists and Quit otherwise; when no selection exists, the page interrupt hook may cancel work before the quit request.", "Classification uses key context and gives the page interrupt hook the no-selection path.")
add([891], "shell-routing-and-page-identity", "Ctrl+C and Ctrl+Q require the exact Control chord.", "Control quit chords require an exact modifier match.")
add([892], "shell-routing-and-page-identity", "The configured letter may be plain or Shifted, with no other modifiers.", "Quit letters accept only no modifiers or Shift.")
add([1012], "shell-routing-and-page-identity", "A dirty page blocks the letter and may show `dirty_letter_note`.", "The optional dirty-letter note reports a blocked quit letter.")
add([901, 914], "shell-routing-and-page-identity", "For other stakes, `request` keeps the pending leave while the app shows the dialog; report the chosen answer to `resolve`.", "The guard retains the requested leave while the application displays and resolves its dialog.")
add([901], "shell-routing-and-page-identity", "The app renders the requested dialog with its own component and follows the selected kind's action meanings.", "The app supplies the dialog renderer and preserves the selected kind's declared action meaning.")
add([903, 907], "shell-routing-and-page-identity", "`SaveDiscardCancel` offers Cancel, Discard, and a focused Save action", "SaveDiscardCancel offers its declared actions and focuses Save.")
add([903, 908], "shell-routing-and-page-identity", "`DiscardOnly` offers Cancel and Discard", "DiscardOnly offers Cancel and Discard, without Save.")
add([903, 909], "shell-routing-and-page-identity", "`StopStay` offers Stay and Stop and leave", "StopStay exposes Stay and Stop-and-leave choices.")
add([900, 907, 908, 909], "shell-routing-and-page-identity", "A `Resolved::Save` result means save first, then carry out the pending leave; `Resolved::Discard` means abandon unsaved changes, then carry it out.", "The caller performs the selected save or discard before carrying out the leave.")
add([910, 911, 912, 913], "shell-routing-and-page-identity", "Report `First` for Cancel or Stay, `Second` for Discard or Stop and leave, and `Third` for Save only when that kind offers it.", "The reported answer index must match the offered dialog action.")
add([916], "shell-routing-and-page-identity", "A read-only context offers Stay or Discard, not Save.", "Read-only contexts do not offer Save.")
add([917], "shell-routing-and-page-identity", "A new guard starts with Save enabled", "A new leave guard allows the Save action unless changed by the caller.")
add([918], "shell-routing-and-page-identity", "`save(false)` offers Discard instead of Save for unsaved changes", "The caller can disable Save to offer Discard in its place.")
add([919], "shell-routing-and-page-identity", "A clean stake returns the requested leave without a dialog.", "A clean leave request proceeds without a confirmation dialog.")
add([920], "shell-routing-and-page-identity", "Ctrl+Q classifies as Quit when enabled, then the leave guard handles stakes.", "A quit request passes its stakes through the leave guard.")
add([922], "shell-routing-and-page-identity", "`request` keeps the pending leave while the app shows the dialog", "The caller keeps the pending leave while the dialog is open.")
add([923], "shell-routing-and-page-identity", "`resolve` consumes the pending leave; with no pending leave it returns Stay.", "Resolution consumes the pending leave; resolving with no pending path returns Stay.")
add([916], "shell-routing-and-page-identity", "A read-only context offers Stay or Discard, not Save.", "Read-only guard contexts cannot offer Save.")
add([967, 977, 978, 982, 986, 987], "shell-layout-status-and-reports", "The default report puts its heading, subheading, empty state, and body on successive rows, with one column of inset on each side. Its inset and row policy can be changed. A configured minimum detail height controls whether the subheading and empty state are shown; zero means no minimum-height gate. Clip every line to the report area and stop body rows at the allocated bottom edge. A lazy report requests only rows that fit the allocation.", "Report row placement, threshold, clipping, and lazy iteration are source-defined rendering constraints.")
add([988, 994, 1005, 1007, 1008, 1009, 1010, 1013, 1027, 1030, 1031, 1032, 1033, 1034, 1035, 1036, 1038, 1039, 1040, 1043, 1046, 1048, 1049], "shell-routing-and-page-identity", "Ask the active layer first; if it returns `Some`, stop the route even when its response is not consumed. Ask the open menu next; `Some` also stops the route. Run the `priority` tier before page input. With a key-capturing component, give the page a chance before the `keys` tier. Otherwise, run the `keys` tier, then app chrome, then the page. The `keys` tier holds the shell's own bindings. Run `fallback` only after earlier handlers decline.", "Routing is ordered across layer, menu, priority, capture, page, chrome, keys, and fallback, with exact chord matching.")
add([1010, 1020], "shell-routing-and-page-identity", "A text focus or active capture quiets bare-character bindings and `idle_only` actions, including `idle_only` actions on non-character chords.", "Text focus and component capture suppress bare-character bindings; idle_only suppresses its actions for any chord under either owner.")
add([1015, 1016, 1017, 1018], "shell-routing-and-page-identity", "`bind`, `bind_priority`, `bind_fallback`, and `bind_with` put a new binding before peers in that tier; exact-chord lookup uses the first match.", "New bindings take precedence over existing same-tier peers for an exact chord.")
add([1019], "shell-routing-and-page-identity", "`unbind` removes matching chords from all three tiers.", "Unbinding removes the matching exact chord from every shell tier.")
add([1041, 1042, 1047], "shell-routing-and-page-identity", "For a paste, give the open layer the first chance, then the page.", "Paste input goes to an open layer before the page.")
add([1050, 1063, 1069, 1070], "shell-layout-status-and-reports", "The outer composition spec controls only the one-child shell adapter, not the space between shell regions. Keep `ShellSlots` alive while a borrowed shell group is in use. `AppShell::show` resolves the three slots from the framework root and draws each visible leaf once.", "Shell regions share one chrome resolver; the borrowed slots live through use, and visible leaves draw once per show.")
add([1079, 1080, 1081, 1082, 1083], "shell-layout-status-and-reports", "The shared roles are Error → Danger, Success → Success, Normal → primary foreground, and Secondary → secondary foreground.", "Status tones map onto shared theme roles.")
add([1084, 1088, 1089, 1105, 1109], "shell-layout-status-and-reports", "Information expires after five seconds by default; `ttl_ms` changes that limit. Errors do not expire on time. A key, paste, mouse Down, or Secondary press clears an error; Tick, Resize, and other mouse input do not.", "Informational TTL and error-clear events use distinct rules.")
add([1085, 1095, 1110, 1111], "shell-layout-status-and-reports", "A progress message belongs to a job and ends when that job ends. Use a distinct `JobKey` for each live job that must end independently. Setting another message does not end the job.", "Progress messages have independent job ownership; replacing text does not end a job.")
add([1088, 1092, 1093, 1096, 1097, 1099, 1105, 1106, 1107, 1108], "shell-layout-status-and-reports", "Observe the feedback clock before using `set` or `set_with`; the `_at` forms observe it for you.", "Observed-clock and explicit-clock APIs have distinct preconditions.")
add([1106], "shell-layout-status-and-reports", "A key, paste, mouse Down, or Secondary press clears an error; Tick, Resize, and other mouse input do not.", "Only key, paste, Down, and Secondary input clear an error.")
add([759, 762, 763, 764, 765, 766, 772, 773, 774, 775, 776, 778, 779, 781, 789], "shell-layout-status-and-reports", "`compose` preserves that order, appends Tab-next once when enabled and not editing, and filters labels: it rejects recognized Shift-pointer labels and requires recorded delivery for recognized Cmd/Super and non-pointer Alt/Option prefixes.", "Footer composition keeps input order, defines empty/fallback behavior, and gates Tab-next and key labels.")
add([790], "scroll-and-keyboard-sessions", "The shell observes modifier delivery and filters hint labels; the terminal session owns protocol push/pop.", "Modifier hint delivery and terminal keyboard-mode negotiation have different owners.")
add([793, 794, 795, 796, 797, 798], "shell-layout-status-and-reports", "`observe` handles key input only. An explicit refusal of Super or Alt stays in force; otherwise, a matching key input can record delivery. The accessors read cached state; they return false until state is set to delivered or observed. The setters record caller-supplied probe or capability answers. The filter records label policy; it does not prove terminal support or emitted output.", "Modifier observation respects explicit refusal; accessors read cached answers; recognized label prefixes consult delivery state without proving terminal support.")
add([806, 807, 808, 809, 810, 811, 812], "shell-routing-and-page-identity", "Check dropped pages in top-first order and pass their stakes through the leave guard before applying a move. A move that drops no page needs no leave confirmation.", "The application applies a Nav request only after checking every page it drops.")
add([841, 843, 844, 845, 846, 847, 849], "shell-routing-and-page-identity", "Crumb segments are built from the active stack, root first. Use their emitted child IDs when registering hits. Every segment except the final one links back.", "Breadcrumb data and hit registration follow root-first segments and only nonfinal links.")
add([850, 852], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents.", "The page drains owner-scoped input and lifecycle intents during update.")
add([851], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents.", "The Page trait includes its update lifecycle.")
add([851, 853], "shell-layout-status-and-reports", "Draw into the shell body region.", "Page draw uses the body region supplied by shell composition.")
add([851, 855], "shell-layout-status-and-reports", "A Page label names its breadcrumb segment.", "The page label supplies its segment in the navigation trail.")
add([851, 856], "shell-layout-status-and-reports", "Supply the page label and `Stakes` used by chrome, footer, and leave guard.", "Page metadata supplies the shell's label and leave-guard stakes.")
add([864], "shell-layout-status-and-reports", "The `page_leaf` adapter borrows the page for the returned leaf's use.", "The borrowed page must outlive the composition leaf use.")


def main() -> None:
    parser = argparse.ArgumentParser(description="Build the T37 support-cohort evidence record.")
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--clean-root", type=Path, required=True)
    parser.add_argument("--crosswalk", type=Path, default=Path("docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json"))
    parser.add_argument("--doc", type=Path, default=Path("docs/contracts/foundation-support.md"))
    parser.add_argument("--output", type=Path, default=Path("docs/implementation/pr38-final/results/t37-api-support/foundation-support-disposition.json"))
    args = parser.parse_args()

    source_map_path = args.source_root / "source-api-prose.json"
    disposition_path = args.source_root / "source-api-prose-disposition.json"
    removal_path = args.source_root / "source-removal.json"
    source_map_bytes = source_map_path.read_bytes()
    disposition_bytes = disposition_path.read_bytes()
    removal_bytes = removal_path.read_bytes()
    assert sha256(source_map_bytes) == SOURCE_MAP_SHA
    assert sha256(disposition_bytes) == SOURCE_DISPOSITION_SHA
    source_map = json.loads(source_map_bytes)
    disposition = json.loads(disposition_bytes)
    disposition_by_index = {row["source_map_record"]: row for row in disposition["records"]}
    removal = json.loads(removal_bytes)
    crosswalk = json.loads(args.crosswalk.read_text())
    support_text = args.doc.read_text()
    assert source_map["record_count"] == len(source_map["records"]) == 1122
    assert len(disposition["records"]) == len(source_map["records"])

    removed_paths = set(removal.get("removed_source_paths", []))
    removed_paths.update(removal.get("removed_test_target_paths_from_inventory", []))
    cohort_prefixes = (
        "crates/termrock-shell/",
        "crates/termrock-theme/",
        "crates/termrock-collections/",
        "crates/termrock-terminal/",
    )
    selected = [row for row in crosswalk["records"] if row["path"].startswith(cohort_prefixes)]
    assert len(selected) == 496
    rows: list[dict[str, Any]] = []
    counts: Counter[str] = Counter()
    source_cache: dict[str, list[str]] = {}
    source_scope_cache: dict[str, dict[int, tuple[str, ...]]] = {}
    clean_scope_cache: dict[str, dict[int, tuple[str, ...]]] = {}
    unresolved: list[int] = []
    clean_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=args.clean_root, text=True).strip()
    clean_tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=args.clean_root, text=True).strip()

    for original in selected:
        index = original["source_map_record"]
        source_record = source_map["records"][index]
        lexical = disposition_by_index[index]
        assert original["id"] == sha256(canonical(source_record))
        assert original["path"] == source_record["path"] == lexical["path"]
        assert original["source_sha256"] == source_record["source_sha256"] == lexical["source_sha256"]
        assert original["line_range"] == {"start": source_record["start_line"], "end": source_record["end_line"]}
        raw = source_bytes(args.repo, original["path"])
        assert sha256(raw) == original["source_sha256"]
        if original["path"] not in source_cache:
            source_cache[original["path"]] = raw.decode("utf-8").splitlines()
            source_scope_cache[original["path"]] = lexical_scopes(source_cache[original["path"]])
        item, item_name, item_signature, item_kind, declaration_line = declaration(original["path"], source_cache[original["path"]], source_record["start_line"], source_record["end_line"])
        comment = "\n".join(source_record["text"])
        clean_path = args.clean_root / original["path"]
        clean_lines = clean_path.read_text().splitlines() if clean_path.exists() else []
        if clean_path.exists() and original["path"] not in clean_scope_cache:
            clean_scope_cache[original["path"]] = lexical_scopes(clean_lines)
        source_scope = source_scope_cache[original["path"]].get(declaration_line, ()) if declaration_line else ()
        clean_matches = []
        if clean_path.exists() and item_signature is not None:
            clean_scopes = clean_scope_cache[original["path"]]
            clean_matches = [
                line_number
                for line_number, line in enumerate(clean_lines, 1)
                if line.strip() == item_signature and clean_scopes.get(line_number, ()) == source_scope
            ]
        item_survives = clean_path.exists() and (
            (item_kind == "module" and clean_path.is_file())
            or bool(clean_matches)
        )
        item_survival_evidence = (
            f"`{item_signature}` occurs at clean line(s) {clean_matches} inside lexical scope `{list(source_scope)}` at clean tree {clean_commit}"
            if clean_matches
            else f"module file exists at clean tree {clean_commit}"
            if item_kind == "module" and clean_path.is_file()
            else "attached declaration line is absent from the accepted clean-source tree"
        )
        category = lexical["category"]
        destination = None
        excerpt = None
        required_clause: Any = None
        clauses: list[dict[str, str]] = []

        if category in ("removed_with_deleted_test_or_fixture_source", "inside_cfg_test_scope", "attached_to_test_only_scope") or original["path"] in removed_paths:
            semantic = "retired_test_only_provenance"
            destination = None
            excerpt = None
            required_clause = None
            status = "removed_with_t37_test_scope"
            reason = (
                f"This exact comment group belonged to `{original['path']}` at lines "
                f"{source_record['start_line']}-{source_record['end_line']}, which the accepted T37 source map classifies as "
                f"{category}. T37 removes that test/fixture scope; the source text and hashes are retained here as history, "
                "not as an active API or proof obligation."
            )
        elif index in (791, 792):
            semantic = "internal_implementation_description"
            destination = None
            excerpt = None
            required_clause = None
            status = "retained_source_item"
            reason = (
                f"This group documents the private `{item_name}` `AtomicU8` encoding for one modifier. "
                "The numbers 0 through 3 are not accepted or returned by the public API; callers use the "
                "boolean delivery accessors and setters. Keep the encoding as internal implementation detail, "
                "not a public caller contract."
            )
        elif index in INTERNAL_INDICES:
            semantic = "internal_implementation_description"
            destination = None
            excerpt = None
            required_clause = None
            status = "retained_source_item"
            if index == 423:
                reason = "`ScrollState.context` is private storage for reveal-span context. The caller-visible context behavior is stated for `reveal_span` in the maintained scroll contract; the field does not add a separate public rule."
            elif index == 424:
                reason = "`ScrollState.exact` is a private one-shot flag for pointer-driven exact reveal. The caller-visible no-context behavior is stated for `ensure_visible`; the field does not add a separate public rule."
            elif index == 425:
                reason = "`ScrollState.followed` is private deduplication state. The caller-visible repeated-cursor no-op is stated for `FollowCursor::OnMove`; callers do not set or retain this tuple."
            elif index == 426:
                reason = "`ScrollState.paging` is private drag state. The caller-visible Page-versus-Jump track behavior is stated in the maintained scroll contract; callers do not set this flag."
            elif index == 376:
                reason = "`bright` is a private helper for the public `legible_mark` fallback sequence, which is stated in the maintained theme contract. It is not a callable public color conversion API."
            elif index == 740:
                reason = "`Browse` is a private screen type inside the retained `shell_consumer` example. The comment labels that example type; it adds no external caller precondition, public lifecycle, or API rule. T35 owns complete current example guidance."
            else:
                reason = "`TextReport::band` is a private geometry helper. The public report row, inset, clipping, and bounded-iteration rules are stated in the maintained report contract; callers do not invoke this helper."
        elif index in RULES:
            decisions = RULES[index]
            semantic = decisions[0]["semantic_disposition"]
            assert all(decision["semantic_disposition"] == semantic for decision in decisions)
            for decision in decisions:
                assert decision["supporting_excerpt"] in support_text, f"support excerpt absent for {index}: {decision['supporting_excerpt']}"
            clauses = [{
                "destination": decision["destination"],
                "supporting_excerpt": decision["supporting_excerpt"],
                "required_clause": decision["required_clause"],
            } for decision in decisions]
            destination = decisions[0]["destination"]
            excerpt = decisions[0]["supporting_excerpt"]
            required_clause = [decision["required_clause"] for decision in decisions]
            if index in FOOTER_PIN_INDICES:
                semantic = "unresolved_source_contract_conflict"
                status = "retained_source_conflict_open"
                if index in (762, 763):
                    reason = (
                        f"The comment attached to `{item}` promises that HINT_PINNED survives an error. "
                        "In c817 `Footer::compose` assigns a dirty matching hint HINT_PINNED, then the "
                        "Role::Danger pass assigns HINT_NORMAL to every retained hint (lines 179–190). "
                        "The comment and assignment order conflict. T37-FOOTER-PIN-CONTRACT-01 keeps the "
                        "promise as an open policy requirement for T19-FOOTER-PIN-01; T37 makes no behavior change."
                    )
                else:
                    reason = (
                        f"The comment attached to `{item}` describes the error action as demotion. "
                        "The current `compose` implementation normalizes every retained hint to HINT_NORMAL, "
                        "without removing or reordering it; HINT_AUTO changes from 0 to 1. The pinned-survival "
                        "promise in the adjacent source comments conflicts with this assignment order. "
                        "T37-FOOTER-PIN-CONTRACT-01 leaves the precedence decision to T19-FOOTER-PIN-01."
                    )
            elif index in (378, 379, 380, 381):
                semantic = "retained_public_compatibility_data"
                status = "retained_public_item"
                reason = (
                    f"The source attaches `{item}` to the exact array `{source_record['text'][0].strip()}`. "
                    "T37-THEME-DATA-01 retains its declared values and order as existing ungated public data; "
                    "the maintained excerpt makes no test-coverage, production-consumer, or permanent-exhaustiveness claim."
                )
            elif index in (758, 866):
                semantic = "historical_or_internal_description"
                reason = (
                    f"This module group at `{original['path']}` records the PR #26 port and ownership adaptations. "
                    "Current caller and lifecycle rules from the module are separately mapped to maintained sections; "
                    "the port lineage and removed integration names are historical provenance, not current contracts."
                )
            elif index in VISIT_KEY_INDICES:
                semantic = "unresolved_source_contract_conflict"
                status = "retained_source_conflict_open"
                reason = (
                    f"The comment attached to `{item}` promises distinct revisit identity or a monotonic visit counter. "
                    "At c817, one shared u64 counter advances with wrapping_add and key() uses zero as its absent-entry "
                    "fallback. T37-VISIT-KEY-01 records the current source facts and leaves collision-safe exhaustion "
                    "behavior open as T19-VISIT-KEY-01; T37 makes no source change and no failure was executed."
                )
            else:
                status = "retained_source_item" if item_survives else "source_item_not_found_in_accepted_output"
                reason = f"The source group states these specific clauses for {item}: {'; '.join(required_clause)} The quoted maintained excerpts record them."
                if not item_survives:
                    unresolved.append(index)
        else:
            semantic = "historical_or_internal_description" if index in (417, 420, 739, 741, 802) else "descriptive_api_reference"
            destination = None
            excerpt = None
            required_clause = None
            if index == 417:
                reason = "This comment labels the PR #26 origin for FollowCursor. The current follow behavior is separately recorded for groups 418, 419, and 433; this port citation adds no caller rule."
            elif index == 420:
                reason = "This comment labels the PR #26 origin for TrackClick. The current Jump/Page and drag behavior is separately recorded for groups 421, 422, and 434; this port citation adds no additional caller rule."
            elif index == 739:
                reason = "This module comment identifies `shell_consumer.rs` as an executable public-API example and lists its shell parts. It describes the example's purpose; it does not define an additional shell contract. T01/T35 own complete example guidance."
            elif index == 741:
                reason = "This module comment records the PR #26 port and why the menu command map and dialog inset live with other component owners. The active shell geometry contract is recorded separately; the source-port history is not a new caller precondition."
            elif index == 802:
                reason = "This module comment records the PR #26 navigation port and the `Option`/`Crumb` adaptation. The externally relevant empty-root panic and optional active-stack behavior are separately recorded for groups 815 and 816; the port lineage itself adds no caller rule."
            else:
                short = normalized(source_record["text"])
                if item.startswith("module documentation"):
                    reason = f"This module group describes {item} in source words `{short[:240]}`. Its caller-facing clauses are either mapped by a separate required row or it gives module context; the full API/reference text remains T01/T35 work."
                else:
                    reason = (
                        f"This group is attached to {item} and says `{short[:300]}`. It names or explains that member's value, "
                        "purpose, example, or accessor result; it adds no separate caller precondition, sequencing/failure rule, "
                        "identity/selection requirement, callback/borrow limit, or bounded-work clause. T01/T35 own full API/reference prose."
                    )
            status = "retained_source_item" if item_survives else "source_item_not_found_in_accepted_output"
            if not item_survives and category == "surviving_production_source":
                unresolved.append(index)

        rows.append({
            "id": original["id"],
            "source_map_record": index,
            "path": original["path"],
            "line_range": original["line_range"],
            "source_sha256": original["source_sha256"],
            "text_sha256": original["text_sha256"],
            "source_text": comment,
            "attached_item": item,
            "attached_item_kind": item_kind,
            "attached_item_signature": item_signature,
            "attached_item_scope": list(source_scope),
            "clean_declaration_match_lines": clean_matches,
            "item_survival_evidence": item_survival_evidence,
            "attachment_status": status,
            "lexical_category": category,
            "semantic_disposition": semantic,
            "destination": destination,
            "required_clauses": clauses if index in RULES else [],
            "decision_ids": (
                [FOOTER_PIN_DECISION] if index in FOOTER_PIN_INDICES
                else [VISIT_KEY_DECISION] if index in VISIT_KEY_INDICES
                else [THEME_DATA_DECISION] if index in (378, 379, 380, 381)
                else []
            ),
            "required_clause": required_clause,
            "supporting_excerpt": excerpt,
            "specific_reason": reason,
        })
        counts[semantic] += 1

    assert not unresolved, f"attached production items not found after T37: {unresolved}"
    output = {
        "schema": "t37.foundation-api-prose-cohort-disposition.v1",
        "task_id": "T37",
        "cohort_id": "T37-API-PROSE-COHORT-C",
        "status": "PENDING_INDEPENDENT_REVIEW",
        "owner": {"agent_id": "/root/t00_import_rules", "model": "gpt-6-luna", "effort": "max"},
        "work_input_commit": WORK_INPUT,
        "source_input_commit": SOURCE_INPUT,
        "product_source_commit": PRODUCT,
        "accepted_foundation_source_commit": FOUNDATION_OUTPUT,
        "clean_source_commit": clean_commit,
        "clean_source_tree": clean_tree,
        "source_inventory": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json",
            "sha256": sha256(source_map_bytes),
            "record_count": len(source_map["records"]),
        },
        "source_disposition_inventory": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json",
            "sha256": sha256(disposition_bytes),
            "record_count": len(disposition["records"]),
        },
        "source_removal_record": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-removal.json",
            "sha256": sha256(removal_bytes),
            "accepted_source_commit": FOUNDATION_OUTPUT,
            "removed_source_path_count": len(removal.get("removed_source_paths", [])),
        },
        "maintained_documents": {
            path: sha256((args.repo / path).read_bytes())
            for path in (
                "docs/contracts/authoring.md",
                "docs/contracts/foundation-support.md",
                "docs/contracts/theming.md",
            )
        },
        "decision": {
            "id": "T37-API-PROSE-01",
            "reviewer": "/root/t37_components_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "required": "Preserve non-obvious caller preconditions, lifecycle/failure semantics, identity/revision/selection rules, callback purity/borrow limits, and bounded-work contracts in maintained Markdown. Give every source group an exact maintained destination or specific descriptive/internal/redundant/retired-test reason.",
        },
        "additional_decisions": [{
            "id": THEME_DATA_DECISION,
            "reviewer": THEME_DATA_REVIEWER,
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "disposition": "Retain COLOR_LEVELS, FOCUS_BAR_POLICIES, GUTTER_CELLS, BACKDROP_SELECTIONS and re-exports as ungated public compatibility data. Preserve only their current declared values/order; make no coverage, production-consumer, or permanent-exhaustiveness claim.",
            "record_indices": [378, 379, 380, 381],
        }, {
            "id": FOOTER_PIN_DECISION,
            "reviewer": FOOTER_PIN_REVIEWER,
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-shell/src/footer.rs`, source SHA-256 `6c8222660ea4555b84479b820e9001def3e765bc54fffb0e560eef02e17bc69f`",
            "disposition": "Document current `compose` priority assignment and retain the pinned-survives-error promise as OPEN T19-FOOTER-PIN-01. No T37 code change or behavior proof is authorized.",
            "record_indices": sorted(FOOTER_PIN_INDICES),
        }, {
            "id": VISIT_KEY_DECISION,
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-shell/src/nav.rs:12-14,53-56,88-93,124-126`",
            "disposition": "Correct current identity wording and keep the distinct-revisit promise OPEN as T19-VISIT-KEY-01. No source change or behavior proof is authorized.",
            "record_indices": sorted(VISIT_KEY_INDICES),
        }, {
            "id": "T37-NAV-STATE-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-core/src/list.rs:346-371,419-444`",
            "disposition": "Describe TypeAheadPrecedence and NavProfile as current-thread state, with per-thread Shortcuts/Desktop defaults. Make no process-wide, cross-thread, automatic-restore, or runtime-instance-isolation promise.",
            "maintained_destination": "docs/contracts/foundation-support.md#shell-routing-and-page-identity",
            "record_indices": [],
        }, {
            "id": "T37-CANVAS-WIDTH-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-runtime/src/author/canvas.rs:1-31` and `crates/termrock-runtime/src/ui/paint.rs:98-110`",
            "disposition": "Correct current Canvas wording: text_width counts Unicode scalar values and the grapheme painter stops when a cluster cannot fit. Keep exact display-cell width and following-text-run positioning OPEN as T10-CANVAS-WIDTH-01.",
            "maintained_destination": "docs/contracts/authoring.md#component-author-surfaces",
            "record_indices": [],
        }],
        "open_obligations": [{
            "id": "T19-VISIT-KEY-01",
            "task": "T19",
            "status": "OPEN",
            "primary_owner": "termrock-shell owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "The source comments promise distinct keys on revisits and a monotonic counter. At c817, one shared u64 counter advances with wrapping_add; key() uses zero when no stack entry exists. Source arithmetic establishes a finite identity limit, but no collision, wrap, or failure was executed.",
            "source_record_indices": [803, 813],
            "required_outcome": "Prevent ambiguous page/focus identity at exhaustion or explicitly migrate the key contract before claiming unbounded distinct visits.",
            "future_case_scope": ["ordinary push/replace/reset/back/tab identity", "near-limit allocation", "repeated/wrapped/zero identity avoidance", "failure before page/stack/focus mutation if allocation is refused"],
            "limits": "T37 changes no source and adds no tests. T38/T19 must approve a bounded public-production-path way to reach exhaustion; no private test setter, mock, or huge loop is authorized by this record.",
        }, {
            "id": "T19-FOOTER-PIN-01",
            "task": "T19",
            "status": "OPEN",
            "primary_owner": "T19 footer composition and hint policy owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T12 downstream HintBar rendering checks"],
            "source_fact": "At fixed c817, `Footer::compose` filters and preserves the selected hint order, applies a dirty matching-key pin at lines 179–185, and then assigns HINT_NORMAL to every remaining hint on Role::Danger at lines 186–190. Source comment groups 762–763 promise that HINT_PINNED survives an error, which conflicts with that order. The error pass does not remove or reorder hints; HINT_AUTO (0) becomes HINT_NORMAL (1).",
            "required_outcome": "T19 must resolve pin/error precedence and review compatibility before any behavior change. Keep the source promise and current assignment order as separate facts until then.",
            "future_case_scope": ["explicit pinned, normal, auto, and custom priorities", "dirty with matching, nonmatching, and absent pin", "danger, non-danger, and absent status", "error entry and clear while dirty", "top-layer and fallback hints, Tab-next, and delivery filtering", "retained hint order and labels", "downstream visible priority at narrow width"],
            "limits": "No case executed. T37 adds no test, harness, or architecture and changes no source. Product behavior remains NOT VERIFIED.",
        }, {
            "id": "T10-CANVAS-WIDTH-01",
            "task": "T10",
            "status": "OPEN",
            "primary_owner": "runtime Canvas owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "At c817, Canvas::text_width counts Unicode scalar values. The Ui grapheme painter stops when a cluster cannot fit; it does not skip that cluster to admit later text. This is source inspection only.",
            "required_outcome": "Resolve exact display-cell width and following-text-run positioning through an approved runtime design.",
            "future_case_scope": ["ASCII", "combining sequences", "wide grapheme at the final available cell", "text following an unpaintable grapheme", "clipping and adjacent text runs"],
            "limits": "No source repair or behavior case was authorized or executed in T37. Product behavior remains NOT VERIFIED.",
        }],
        "review_method": "For every member, compare the exact comment group with the attached source item at c817, the accepted T37 removal classification, and the item in accepted foundation output 2fee. Required rules point to a literal excerpt in maintained Markdown. Every other group has its own source-specific reason or exact retired-scope basis. No keyword/path heuristic assigns semantic disposition.",
        "record_validation": {
            "source_records": "All 496 selected rows preserve their exact source_map_record, id, path, line range, input file SHA-256, and comment-text SHA-256 from the immutable source map.",
            "attachment": "For surviving declarations, the generator matches the attached declaration signature inside the same lexical container scope in the clean source tree. It does not match a symbol elsewhere in the file.",
            "required_prose": "Every required row points to an exact excerpt that appears in the maintained destination. The generator checks presence, not whether the prose proves behavior.",
            "unresolved_items": len(unresolved),
            "selected_tests": 0,
            "behavior_status": "NOT VERIFIED",
            "independent_review": "PENDING",
        },
        "cohort_counts": {"termrock-shell": 375, "termrock-theme": 95, "termrock-collections": 18, "termrock-terminal": 8},
        "disposition_counts": dict(counts),
        "record_count": len(rows),
        "records": rows,
        "limits": [
            "This is a source-prose disposition record, not a product behavior check or visual acceptance.",
            "T01 must inventory retained public items and T35 must write complete current API and user guidance.",
            "The retained theme arrays record current compatibility data only; T38/later reviewed plans own future variant coverage.",
            "Historical evidence and source-pinned output values do not prove current runtime behavior.",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n")
    print(f"wrote {args.output}: {len(rows)} records; {dict(counts)}; sha256 {sha256(args.output.read_bytes())}")


if __name__ == "__main__":
    main()
