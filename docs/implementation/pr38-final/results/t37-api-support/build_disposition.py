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
KEYS_TIER_DECISION = "T19-KEYS-TIER-01"
KEYS_TIER_INDICES = {1005, 1008}
MODAL_DIM_DECISION = "T11-MODAL-DIM-01"
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
                    if text.startswith("\\\r\n", i):
                        line += 1
                        i += 3
                    elif text.startswith("\\\n", i):
                        line += 1
                        i += 2
                    else:
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
        scopes.setdefault(line, tuple(f"{kind}:{name}" for kind, name, _ in frames if kind))
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

add([1114, 1115], "scroll-and-keyboard-sessions", "Negotiation is opt-in; without `enter`, no protocol bytes are written.", "Keyboard-protocol negotiation is opt-in for each session.")
add([1116], "scroll-and-keyboard-sessions", "Each `enter`, `leave`, and `resume` result reports whether that call emitted a sequence. `Ok(true)` means a sequence was emitted. A no-op returns `Ok(false)` without writing or flushing. Emitted sequences are flushed. The boolean does not report enabled or pushed state.", "`enter` returns `Ok(true)` only after emitting and flushing a sequence; a no-op returns `Ok(false)`, distinct from enabled or pushed state.")
add([1117], "scroll-and-keyboard-sessions", "Each `enter`, `leave`, and `resume` result reports whether that call emitted a sequence. `Ok(true)` means a sequence was emitted. A no-op returns `Ok(false)` without writing or flushing. Emitted sequences are flushed. The boolean does not report enabled or pushed state.", "`leave` reports whether it emitted a pop; a no-op returns `Ok(false)` without writing or flushing.")
add([1118], "scroll-and-keyboard-sessions", "Each `enter`, `leave`, and `resume` result reports whether that call emitted a sequence. `Ok(true)` means a sequence was emitted. A no-op returns `Ok(false)` without writing or flushing. Emitted sequences are flushed. The boolean does not report enabled or pushed state.", "`resume` reports whether it emitted a push; a no-op returns `Ok(false)` without writing or flushing.")
add([1116], "scroll-and-keyboard-sessions", "`Auto` uses the caller's capability probe; `On` skips it and `Off` writes no protocol sequence.", "Auto mode requires a caller-provided capability result; enabled and emitted state remain separate.")
add([1119, 1120, 1121], "scroll-and-keyboard-sessions", "`state`, `is_pushed`, and `is_enabled` only read state.", "State accessors do not write terminal protocol bytes.")

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
add([335], "theme-presentation-policy", "`Role` currently has no `#[non_exhaustive]` marker. An exhaustive downstream match may need an update when a later release adds a variant.", "A downstream exhaustive match may need an update when a later release adds a Role variant. This is source-compatibility guidance, not a compiler restriction.")
add([343], "theme-presentation-policy", "`ColorTokens` also has no `#[non_exhaustive]` marker, and its public fields permit complete struct literals today. A later field addition may require an update to those literals. For partial customization, start from a built-in theme's `color` tokens and change selected fields.", "Complete ColorTokens literals may need updates when fields are added. For partial customization, start from built-in theme tokens and change selected fields; current legal literals are not forbidden.")
add([351], "focus-selection-and-gutter-policy", "The theme helpers expose shared policies. Source comments say existing component families still paint through recipe parts until they adopt the helpers.", "Theme helpers are not a claim that all component families use them.")
add([352, 353, 354, 355, 371, 373, 374], "focus-selection-and-gutter-policy", "Focused items use accent ink. An unfocused item uses accent only when its `InactiveSelection` policy is `Accent`; otherwise it uses primary text.", "Focused and inactive selected-item ink follows InactiveSelection policy.")
add([352, 364, 365, 366, 372, 373, 374], "focus-selection-and-gutter-policy", "Under a modal or popup, those marks use the neutral policy unless the explicit backdrop policy selects accent.", "Modal/backdrop selection changes only under the explicit BackdropSelection policy.")
add([356, 357, 358, 359, 360], "focus-selection-and-gutter-policy", "Button bars need focus and focus-visible, and remain visible when disabled. List bars need focus and enabled state at every tier. Choice, Tree, and Steps draw on focus, including disabled rows, at TrueColor, ANSI256, and ANSI16; Mono blanks a disabled row. Tier-sensitive policies read the theme's own capability level.", "The shared focus-bar policies retain family- and tier-specific rules.")
add([361, 362, 363, 370], "focus-selection-and-gutter-policy", "`GutterCell::Inherit` stamps only the focus bar foreground and leaves the rest of the cell to the container. `GutterCell::Fill` stamps an explicit blank in the host background.", "Gutter-cell policy distinguishes inherited container paint from an explicit host-background fill.")
add([367], "focus-selection-and-gutter-policy", "`gutter_symbol` uses `RowBlanked`: it returns the phosphor bar only while the widget is focused and enabled, and otherwise reserves a blank cell.", "The default gutter helper requires focus and an enabled widget.")
add([368, 369], "focus-selection-and-gutter-policy", "Use `focus_bar_symbol` with an explicit family policy when those defaults do not match the widget.", "Caller-selected focus-bar policy determines the symbol and gutter style.")
add([375, 377], "focus-selection-and-gutter-policy", "Choose ink against the background. Try the input, a brighter named-ANSI color, Danger and Success, primary text, then the more legible of primary or on-accent ink. If either color has no known RGB, including `Color::Reset`, the contrast ratio is unknown and the pair does not pass this check.", "Legible-mark fallback order is fixed; unknown RGB never passes the contrast check.")
add([378, 379, 380, 381], "descriptive-and-historical-source-notes", "The retained public arrays expose their declared values and order. `COLOR_LEVELS` is `[TrueColor, Ansi256, Ansi16, Mono]`. `FOCUS_BAR_POLICIES` follows `FocusBarPolicy::ALL` as `[RowBlanked, RowKept, FocusVisible]`. `GUTTER_CELLS` follows `GutterCell::ALL` as `[Inherit, Fill]`. `BACKDROP_SELECTIONS` follows `BackdropSelection::ALL` as `[Neutral, Accent]`.", "Preserve current public array values and order as compatibility data; do not infer coverage or permanent exhaustiveness.")
add([311, 312, 313], "focus-selection-and-gutter-policy", "`CursorStyle::Bar` selects the theme focus gutter. `CursorStyle::Fill` highlights the active row and reserves a blank gutter cell. These are public policy choices; this note does not certify every widget's rendering.", "CursorStyle declares Bar and Fill policies; preserve their stated meanings without a widget-wide rendering claim.")

add([756, 751, 1054], "shell-layout-status-and-reports", "The default shell geometry is one header row, one gap row, one footer row, and no inset. Keep the strip rows while the area can hold them; the body uses the remaining rows and may be empty.", "Chrome splitting preserves strip rows when possible and uses the remainder for the body; defaults are fixed.")
add([762, 763], "shell-layout-status-and-reports", "The source promise that pinned hints survive errors conflicts with this assignment order.", "Preserve the pinned-survival promise and current normalization order as separate source facts until T19 resolves their precedence.")
add([765, 766], "shell-layout-status-and-reports", "`Hint::new` starts at `HINT_AUTO`.", "New hints default to HINT_AUTO unless the caller changes priority.")
add([772], "shell-layout-status-and-reports", "The first non-empty layer hint list wins. An empty list leaves the prior choice or fallback.", "The first non-empty layer list takes precedence; an empty list leaves the previous choice intact.")
add([772], "shell-layout-status-and-reports", "`compose` preserves order and appends Tab-next once when enabled outside edit mode.", "Tab-next is added once only outside edit mode, and compose applies the documented label filter.")
add([772], "shell-layout-status-and-reports", "Filtering records label policy; it does not prove terminal support or emitted output.", "Hint filtering records label policy only; it does not prove terminal support or output.")
add([772], "shell-layout-status-and-reports", "Danger status then sets every remaining priority to `HINT_NORMAL`, including pinned and custom priorities. It keeps hints and their order. `HINT_AUTO` changes from 0 to 1.", "The source error wording is retained as a conflict while current code normalizes every returned priority.")
add([764, 771], "shell-layout-status-and-reports", "Danger status then sets every remaining priority to `HINT_NORMAL`, including pinned and custom priorities. It keeps hints and their order. `HINT_AUTO` changes from 0 to 1.", "The error path assigns a common priority and does not necessarily reduce each prior numeric value.")
add([781], "shell-layout-status-and-reports", "The first non-empty layer hint list wins. An empty list leaves the prior choice or fallback.", "Layer hint folding selects the first non-empty list and uses fallback only when no layer supplies one.")
add([790], "scroll-and-keyboard-sessions", "The shell observes modifier delivery; the terminal session owns protocol push and pop.", "Shell modifier-delivery hints and terminal keyboard-mode protocol have separate owners.")
add([801], "shell-layout-status-and-reports", "The shell supplies routing vocabulary and composes its leaves through the runtime. The runtime owns focus, hit testing, pointer routing, layers, and the update loop.", "Shell/runtime ownership is split; the removed reachability harness is not part of the retained shell surface.")
add([805], "shell-routing-and-page-identity", "`PageKey::of` hashes the caller's value.", "PageKey::of derives page identity from the caller's hashable value.")
add([803, 813], "shell-routing-and-page-identity", "Source comments promise distinct revisit keys and call the counter monotonic. The implementation advances one shared `u64` counter with `wrapping_add(1)`. `key()` uses zero when no entry exists. No non-reuse guarantee exists at exhaustion.", "Preserve the revisit-identity promise as an open requirement; do not claim non-reuse at counter exhaustion.")
add([989, 992], "shell-routing-and-page-identity", "`KeyBinding.priority` is an ordering weight for footer hints. Lower values come first when a consumer sorts hints.", "Preserve the declared lower-first ordering meaning for KeyBinding.priority without claiming current product sorting or rendering.")
add([1024, 1025, 1026], "shell-routing-and-page-identity", "Run the `priority` tier before page input. During capture, offer `page_key` before the `keys` tier. Without capture, run `keys`, then app chrome, then the page.", "Priority is before the page; Keys is the shell tier; Fallback runs after earlier handlers decline.")
add([899, 900], "shell-routing-and-page-identity", "After `Resolved::Save`, save before continuing the leave. After `Resolved::Discard`, abandon changes before continuing.", "Save before continuing the leave; abandon unsaved changes before continuing after Discard.")
add([806], "shell-routing-and-page-identity", "Check dropped pages top-first through the leave guard before applying a move.", "Nav requests are resolved through the quit guard before application.")
add([807], "shell-routing-and-page-identity", "`Push` adds a page above the active top without dropping pages.", "Push shows a new route above the active page without dropping the current stack.")
add([808], "shell-routing-and-page-identity", "`Replace` swaps and drops the old top.", "Replace swaps the active top and returns the dropped page for guard handling.")
add([809, 834], "shell-routing-and-page-identity", "`Reset` replaces the stack and drops every old entry.", "Reset replaces the active stack and drops its old entries.")
add([810], "shell-routing-and-page-identity", "`Back` drops the top only when a non-root page exists.", "Back retains the root entry.")
add([811], "shell-routing-and-page-identity", "`PopTo(d)` keeps `d + 1` entries and drops later entries.", "PopTo retains the requested depth and drops later pages.")
add([812], "shell-routing-and-page-identity", "`Tab` switches stacks without dropping pages.", "Tab selection switches stacks without dropping their pages.")
add([815], "shell-routing-and-page-identity", "`Router::with_tabs` requires a root and panics when roots are empty.", "with_tabs has a nonempty-root precondition and panics on an empty root list.")
add([816], "shell-routing-and-page-identity", "Its optional accessor returns `None` if that invariant fails.", "The active-stack accessor returns None if the active-tab invariant fails.")
add([831], "shell-routing-and-page-identity", "A pop keeps the root.", "Back/pop retains the root entry.")
add([835], "shell-routing-and-page-identity", "`select` returns false for an out-of-range or active tab.", "Tab selection reports false for invalid or no-op selection.")
add([836], "shell-routing-and-page-identity", "Opening a tab creates and activates its root stack.", "Opening selects the new root stack.")
add([837], "shell-routing-and-page-identity", "Closing an invalid or final tab returns no result. Successful close returns its pages and preserves one tab.", "Closing rejects an invalid or final tab and returns removed page ownership while preserving a tab.")
add([838], "shell-routing-and-page-identity", "A move that drops no page needs no confirmation.", "A move with no dropped page does not require a leave guard.")
add([839], "shell-routing-and-page-identity", "Check dropped pages top-first through the leave guard before applying a move.", "Check each dropped page in top-first order before applying the move.")
add([840], "shell-routing-and-page-identity", "Apply confirmed moves after dialog resolution. Results list dropped pages top-first.", "Apply a guarded move after dialog resolution and preserve top-first dropped-page reporting.")
add([842, 848], "shell-routing-and-page-identity", "Pass the number of extra rendered levels to click mapping; those levels shift the clicked index to its `PopTo` depth.", "Breadcrumb click mapping accounts for extra rendered levels.")
add([841, 843, 844, 845, 846, 847], "shell-routing-and-page-identity", "Crumb segments are built from the active stack, root first. Use their emitted child IDs when registering hits. Every segment except the final one links back.", "Breadcrumb labels follow stack order, use emitted child IDs, and expose only nonfinal links.")
add([849], "shell-routing-and-page-identity", "Every segment except the final one links back.", "Only nonfinal breadcrumb segments are links.")
add([850], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents. Use `on_show` for enter work.", "Page updates own queued intents and on_show owns enter work.")
add([852], "shell-layout-status-and-reports", "Drain page-owned intents in `update`. Focus changes, ticks, keys, pastes, pointer events, and layer events arrive through those intents.", "Page event kinds arrive as owner-scoped intents to update.")
add([853], "shell-layout-status-and-reports", "Draw into the shell body region.", "Page drawing uses the body region supplied by the shell.")
add([854], "shell-layout-status-and-reports", "Use the shell-resolved body bounds for layout-relative drawing and registration.", "Layout-relative drawing and hit registration use the shell-provided body bounds.")
add([857], "shell-routing-and-page-identity", "Focus upkeep compares page keys to detect page changes. It focuses the primary control when a page appears.", "Page primary focus is coupled to page appearance and visit identity.")
add([858], "shell-layout-status-and-reports", "appends Tab-next once when enabled outside edit mode", "Tab-next is hidden while the text field owns typing.")
add([859], "shell-layout-status-and-reports", "Report `animating` only while the runtime should keep repainting.", "The animation flag controls continued runtime repainting.")
add([860], "shell-layout-status-and-reports", "Use `on_show` for enter work.", "on_show is the page-enter hook.")
add([861], "shell-routing-and-page-identity", "Without a selection, the page interrupt hook may cancel work before quit.", "The interrupt hook may cancel page work when Ctrl+C has no selection to copy.")
add([855], "shell-routing-and-page-identity", "Crumb segments are built from the active stack, root first.", "A Page label names its segment in the root-first navigation trail.")
add([856], "shell-routing-and-page-identity", "Check dropped pages top-first through the leave guard before applying a move.", "Page stakes are evaluated for each dropped page before navigation applies.")
add([862], "shell-routing-and-page-identity", "Use `replay_key` when a menu or hint represents a key. Newline maps to Enter. Arrows map to arrow keys. Escape maps to Esc. Other characters map to unmodified character keys.", "Replayed menu/hint keys use the defined character-to-key mapping.")
add([863], "shell-layout-status-and-reports", "`rebuild_on_tick` runs the update closure and reports whether the view changed. Use that result to avoid repainting an unchanged view.", "Tick rebuild result reports view change and can avoid unchanged repaint.")
add([865], "shell-layout-status-and-reports", "The `page_leaf` adapter borrows the page for the returned leaf's use.", "The borrowed page must outlive the composition leaf use.")
add([867, 868], "shell-routing-and-page-identity", "A clean stake returns the requested leave without a dialog.", "A clean stake leaves without opening a confirmation dialog.")
add([869, 870], "shell-routing-and-page-identity", "Unsaved stakes record changes at risk.", "The Unsaved stake carries the count of changes at risk.")
add([871, 872], "shell-routing-and-page-identity", "Running stakes name abandoned work.", "The Running stake names the work that stopping abandons.")
add([874], "shell-routing-and-page-identity", "Running work outranks unsaved changes. Unsaved counts add with saturation.", "Combining stakes prioritizes running work and saturates unsaved counts.")
add([876, 881], "shell-routing-and-page-identity", "Ctrl+C requests Copy with a selection and Quit without one.", "A selection changes Ctrl+C from a quit request into a copy action.")
add([877, 884], "shell-routing-and-page-identity", "Enabled Ctrl+Q requests quit through the leave guard.", "Enabled Ctrl+Q asks the leave guard to handle stakes.")
add([879], "shell-routing-and-page-identity", "Text fields receive bare letters while they own typing.", "A text field retains bare-letter input while it owns typing.")
add([880, 885, 893], "shell-routing-and-page-identity", "It quits only without text focus and when the page is clean.", "A dirty page or active text field blocks the configured quit letter.")
add([882, 883, 887], "shell-routing-and-page-identity", "Defaults are Ctrl+C, Ctrl+Q, and `q`. Escape does not quit.", "Preserve the declared default quit chords and letter.")
add([886, 889], "shell-routing-and-page-identity", "Escape quit is opt-in. It applies without text focus after earlier handlers decline.", "Escape may request quit only when enabled, without text focus, and after earlier handlers decline.")
add([888], "shell-routing-and-page-identity", "The configured letter can change or be disabled with `None`.", "The caller may change or remove the configured quit letter.")
add([890], "shell-routing-and-page-identity", "Ctrl+C requests Copy with a selection and Quit without one. Without a selection, the page interrupt hook may cancel work before quit.", "Classification uses selection state and gives the page interrupt hook the no-selection path.")
add([891], "shell-routing-and-page-identity", "Ctrl+C and Ctrl+Q require the exact Control chord.", "Control quit chords require an exact modifier match.")
add([892], "shell-routing-and-page-identity", "The configured letter accepts no modifiers or Shift.", "Quit letters accept only no modifiers or Shift.")
add([1012], "shell-routing-and-page-identity", "A dirty page blocks it and may show `dirty_letter_note`.", "The optional dirty-letter note reports a blocked quit letter.")
add([901, 914], "shell-routing-and-page-identity", "Other requests remain pending while the app displays its dialog. Report the chosen answer to `resolve`.", "The guard retains the requested leave while the application displays and resolves its dialog.")
add([901], "shell-routing-and-page-identity", "The app renders the requested dialog and follows its action meanings.", "The app supplies the dialog renderer and preserves the selected kind's declared action meaning.")
add([903, 907], "shell-routing-and-page-identity", "`SaveDiscardCancel` offers Cancel, Discard, and focused Save.", "SaveDiscardCancel offers its declared actions and focuses Save.")
add([903, 908], "shell-routing-and-page-identity", "`DiscardOnly` offers Cancel and Discard", "DiscardOnly offers Cancel and Discard, without Save.")
add([903, 909], "shell-routing-and-page-identity", "`StopStay` offers Stay and Stop-and-leave.", "StopStay exposes Stay and Stop-and-leave choices.")
add([900, 907, 908, 909], "shell-routing-and-page-identity", "After `Resolved::Save`, save before continuing the leave. After `Resolved::Discard`, abandon changes before continuing.", "The caller performs the selected save or discard before carrying out the leave.")
add([910, 911, 912, 913], "shell-routing-and-page-identity", "Report `First` for Cancel or Stay. Report `Second` for Discard or Stop-and-leave. Report `Third` for Save only when offered.", "The reported answer index must match the offered dialog action.")
add([916], "shell-routing-and-page-identity", "Read-only contexts offer Stay or Discard, not Save.", "Read-only contexts do not offer Save.")
add([917], "shell-routing-and-page-identity", "New guards allow Save.", "A new leave guard allows the Save action unless changed by the caller.")
add([918], "shell-routing-and-page-identity", "`save(false)` offers Discard instead of Save for unsaved changes", "The caller can disable Save to offer Discard in its place.")
add([919], "shell-routing-and-page-identity", "A clean stake returns the requested leave without a dialog.", "A clean leave request proceeds without a confirmation dialog.")
add([920], "shell-routing-and-page-identity", "Enabled Ctrl+Q requests quit through the leave guard.", "A quit request passes its stakes through the leave guard.")
add([922], "shell-routing-and-page-identity", "Other requests remain pending while the app displays its dialog.", "The caller keeps the pending leave while the dialog is open.")
add([923], "shell-routing-and-page-identity", "`resolve` consumes the pending leave; without one, it returns Stay.", "Resolution consumes the pending leave; resolving with no pending path returns Stay.")
add([916], "shell-routing-and-page-identity", "Read-only contexts offer Stay or Discard, not Save.", "Read-only guard contexts cannot offer Save.")
add([967, 977, 978, 982, 986], "shell-layout-status-and-reports", "The default report uses successive rows and one column of inset on each side. Callers may change the inset and row policy. Minimum detail height controls subheading and empty-state visibility; zero disables that gate. Lines clip to the report area. Body rows stop at its bottom edge. `draw_rows` streams the supplied iterator without collecting a per-frame vector. It requests only rows that fit the allocation.", "Report geometry, threshold, clipping, bottom edge, and bounded lazy iteration are source-defined rendering constraints.")
add([987], "shell-layout-status-and-reports", "`draw_rows` streams the supplied iterator without collecting a per-frame vector. It requests only rows that fit the allocation.", "draw_rows must preserve iterator streaming and the no-per-frame-vector constraint while requesting only rows that fit.")
add([988], "shell-routing-and-page-identity", "Run the `priority` tier before page input. During capture, offer `page_key` before the `keys` tier. Without capture, run `keys`, then app chrome, then the page.", "The module documents priority, capture, and ordinary routing order; the page-versus-keys source conflict remains separately open.")
add([988], "shell-routing-and-page-identity", "Runtime focus handles Tab and BackTab before app handlers. Shell focus actions remain available for custom chords; default Tab bindings do not move focus.", "The module documents that runtime focus owns default Tab movement while shell focus actions remain available on custom chords.")
add([1005], "shell-routing-and-page-identity", "Run the `priority` tier before page input. During capture, offer `page_key` before the `keys` tier. Without capture, run `keys`, then app chrome, then the page. This order conflicts with the source promise that `keys` runs after the page yields. Keep T19-KEYS-TIER-01 open.", "The three-tier comment's page-order promise conflicts with current capture and ordinary routing; keep that policy open under T19-KEYS-TIER-01.")
add([1005], "shell-routing-and-page-identity", "The `keys` tier holds shell bindings. Run `fallback` only after earlier handlers decline.", "The keys tier holds shell bindings; fallback runs only after earlier handlers decline.")
add([1005], "shell-routing-and-page-identity", "Text focus or capture quiets bare-character bindings and `idle_only` actions, including actions on non-character chords.", "Text focus or capture suppresses bare-character bindings; idle_only also suppresses its actions on non-character chords.")
add([1007], "shell-routing-and-page-identity", "Run the `priority` tier before page input.", "The priority tier runs before page input.")
add([1008], "shell-routing-and-page-identity", "This order conflicts with the source promise that `keys` runs after the page yields. Keep T19-KEYS-TIER-01 open.", "The source comment promises the keys tier runs after page decline; current order differs, so T19-KEYS-TIER-01 remains open.")
add([1009], "shell-routing-and-page-identity", "Run `fallback` only after earlier handlers decline.", "Fallback runs only after earlier handlers decline.")
add([1010], "shell-routing-and-page-identity", "Text focus or capture quiets bare-character bindings and `idle_only` actions, including actions on non-character chords.", "Text focus or capture suppresses bare-character bindings; idle_only also suppresses its actions on non-character chords.")
add([1027], "shell-routing-and-page-identity", "The app owns layers, menus, pages, and commands. The shell supplies its key tiers and quit vocabulary.", "The host supplies app-owned layers, menus, pages, and commands; the shell supplies its tiers and quit vocabulary.")
add([1030], "shell-routing-and-page-identity", "Ask the active layer first. A consumed response ends routing. An unconsumed `Some` also stops ordinary page and key routing. Before returning it, an enabled modified quit chord can still request quit.", "The active layer answers first; its response ends ordinary routing, with the documented modified-quit exception before return.")
add([1031], "shell-routing-and-page-identity", "Ask the open menu next; any `Some` ends routing.", "The open menu answers after the layer; any Some ends routing.")
add([1032], "shell-routing-and-page-identity", "During capture, offer `page_key` before the `keys` tier.", "When a component captures keys, the page gets the documented chance before the keys tier.")
add([1033], "shell-routing-and-page-identity", "Text focus or capture quiets bare-character bindings and `idle_only` actions, including actions on non-character chords.", "Text focus quiets bare-character bindings and idle_only actions, including non-character chords.")
add([1035], "shell-routing-and-page-identity", "It quits only without text focus and when the page is clean. A dirty page blocks it and may show `dirty_letter_note`.", "Quit letters require no text focus and a clean page; a dirty page can show the configured note.")
add([1038], "shell-routing-and-page-identity", "App chrome receives non-printable keys.", "The app-chrome handler receives non-printable key input.")
add([1039], "shell-routing-and-page-identity", "During capture, offer `page_key` before the `keys` tier. Without capture, run `keys`, then app chrome, then the page.", "Ordinary routing reaches the page after keys and chrome; capture gives the page a chance before keys.")
add([1040], "shell-routing-and-page-identity", "Escape asks the top layer, then tries shell Back; it does not quit.", "Escape first asks the top layer and then invokes shell Back without requesting quit.")
add([1048], "shell-routing-and-page-identity", "Ask the active layer first. A consumed response ends routing. An unconsumed `Some` also stops ordinary page and key routing. Before returning it, an enabled modified quit chord can still request quit. Ask the open menu next; any `Some` ends routing. Run the `priority` tier before page input.", "Key routing checks the active layer, then menu, then priority before the remaining route.")
add([1048], "shell-routing-and-page-identity", "On parity, the runtime drops hover state on every key press before app routing; the router does no hover work.", "On parity, key input clears hover in the runtime before routing; the router does not handle hover.")
add([994], "shell-routing-and-page-identity", "Match the exact key and modifier chord.", "Lookup matches only the exact key and modifier chord.")
add([1013], "shell-routing-and-page-identity", "The standard shell vocabulary includes F1 help, menu, Ctrl+PgDn/PgUp, Tab focus actions, and Escape back.", "The standard shell vocabulary includes the documented help, menu, page, focus, and Escape actions.")
add([1018], "shell-routing-and-page-identity", "`bind_with` inserts the caller's supplied binding, including its priority, at the front of the selected tier.", "bind_with preserves the caller-supplied footer-hint priority while inserting at the front of the requested tier.")
add([1034], "shell-routing-and-page-identity", "Without a selection, the page interrupt hook may cancel work before quit.", "Without a selection, Ctrl+C offers the page interrupt hook before a quit request.")
add([1036], "shell-routing-and-page-identity", "Ctrl+C requests Copy with a selection and Quit without one.", "Selection state changes Ctrl+C from Quit to Copy.")
add([1043], "shell-routing-and-page-identity", "Enabled Ctrl+Q requests quit through the leave guard.", "The quit path uses the leave guard and opens its dialog when required.")
add([1046], "shell-routing-and-page-identity", "`KeyRouter::input` routes keys and pastes; it ignores other event kinds.", "Input dispatch routes keys and pastes through their handlers and ignores other event kinds.")
add([1049], "shell-routing-and-page-identity", "Use `replay_key` when a menu or hint represents a key. Newline maps to Enter. Arrows map to arrow keys. Escape maps to Esc. Other characters map to unmodified character keys.", "Menu replay maps its represented character to the documented key and routes that key normally.")
add([1010, 1020], "shell-routing-and-page-identity", "Text focus or capture quiets bare-character bindings and `idle_only` actions, including actions on non-character chords.", "Text focus and component capture suppress bare-character bindings; idle_only suppresses its actions for any chord under either owner.")
add([1015, 1016, 1017, 1018], "shell-routing-and-page-identity", "New bindings go before peers in their tier; exact lookup uses the first matching chord.", "New bindings take precedence over existing same-tier peers for an exact chord.")
add([1019], "shell-routing-and-page-identity", "`unbind` removes matches from all tiers.", "Unbinding removes the matching exact chord from every shell tier.")
add([1041, 1042, 1047], "shell-routing-and-page-identity", "Paste goes to the open layer, then the page.", "Paste input goes to an open layer before the page.")
add([1050, 1063, 1069, 1070], "shell-layout-status-and-reports", "The outer composition spec controls only the one-child shell adapter, not the space between shell regions. Keep `ShellSlots` alive while a borrowed shell group is in use. `AppShell::show` resolves the three slots from the framework root and draws each visible leaf once.", "Shell regions share one chrome resolver; the borrowed slots live through use, and visible leaves draw once per show.")
add([1079, 1080, 1081, 1082, 1083], "shell-layout-status-and-reports", "Shared roles map Error to Danger, Success to Success, Normal to primary foreground, and Secondary to secondary foreground.", "Status tones map onto shared theme roles.")
add([1084, 1088, 1089, 1105, 1109], "shell-layout-status-and-reports", "Information expires after five seconds by default; `ttl_ms` changes that limit. Errors do not expire on time. Key, paste, mouse Down, or Secondary input clears an error. Tick, Resize, and other mouse input do not.", "Informational TTL and error-clear events use distinct rules.")
add([1085, 1095, 1110, 1111], "shell-layout-status-and-reports", "A progress message belongs to a job and ends with that job. Use distinct `JobKey` values for jobs that end independently. Replacing text does not end the job.", "Progress messages have independent job ownership; replacing text does not end a job.")
add([1088, 1092, 1093, 1096, 1105, 1107, 1108], "shell-layout-status-and-reports", "Observe the feedback clock before `set` or `set_with`; `_at` forms observe it for you.", "Observed-clock and explicit-clock APIs have distinct preconditions.")
add([1097], "shell-layout-status-and-reports", "`clear` removes the current message and returns true only when one existed.", "clear removes the current message and returns true exactly when one existed.")
add([1099], "shell-layout-status-and-reports", "`entry` returns current text, tone, and payload, or `None`.", "entry queries the current complete message without mutation and returns None when empty.")
add([1106], "shell-layout-status-and-reports", "Key, paste, mouse Down, or Secondary input clears an error. Tick, Resize, and other mouse input do not.", "Only key, paste, Down, and Secondary input clear the error.")
add([1106], "shell-layout-status-and-reports", "`on_input` returns whether the line changed; callers fold that value into `Response`.", "on_input returns its changed flag so callers can fold the change into Response.")
add([759, 762, 763, 764, 765, 766, 772, 773, 774, 775, 776, 778, 779, 781, 789], "shell-layout-status-and-reports", "`compose` preserves order and appends Tab-next once when enabled outside edit mode.", "Footer composition keeps input order, defines empty/fallback behavior, and gates Tab-next and key labels.")
add([790], "scroll-and-keyboard-sessions", "The shell observes modifier delivery; the terminal session owns protocol push and pop.", "Modifier hint delivery and terminal keyboard-mode negotiation have different owners.")
add([793, 795], "shell-layout-status-and-reports", "Accessors read cached state and return false before delivery or observation.", "Super and Alt accessors return cached delivery state and remain false before an observation or supplied answer.")
add([794, 796], "shell-layout-status-and-reports", "Setters record caller-provided probe or capability answers.", "The Super and Alt setters record caller-supplied probe or capability answers.")
add([797], "shell-layout-status-and-reports", "Explicit Super or Alt refusal remains in force. Otherwise, matching key input may record delivery.", "Observed matching input can record delivery unless the caller explicitly refused that modifier.")
add([798], "shell-layout-status-and-reports", "It rejects recognized Shift-pointer labels. Cmd/Super and non-pointer Alt/Option labels require recorded delivery.", "Label filtering rejects Shift-pointer labels and requires recorded modifier delivery.")
add([798], "shell-layout-status-and-reports", "Filtering records label policy; it does not prove terminal support or emitted output.", "Label filtering describes label policy only; it does not prove terminal support or emitted output.")
add([806, 807, 808, 809, 810, 811, 812], "shell-routing-and-page-identity", "Check dropped pages top-first through the leave guard before applying a move. A move that drops no page needs no confirmation.", "The application applies a Nav request only after checking every page it drops.")
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
            elif index in KEYS_TIER_INDICES:
                semantic = "unresolved_source_contract_conflict"
                status = "retained_source_conflict_open"
                reason = (
                    f"The comment attached to `{item}` promises the shell `keys` tier is answered after the page declines. "
                    "At fixed c817, normal routing runs `keys` before chrome and page; capture routing offers the page "
                    "before `keys`. An unconsumed layer response also has a modified-quit exception before its return. "
                    "T37-SUPPORT-R1 records the conflict and leaves routing policy open as T19-KEYS-TIER-01; no behavior "
                    "case or source change is claimed."
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
                else [KEYS_TIER_DECISION] if index in KEYS_TIER_INDICES
                else [MODAL_DIM_DECISION] if index == 322
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
            "local_path_exists": (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json").is_file(),
            "integration_status": "integrated" if (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json").is_file() else "pending_integration",
        },
        "source_disposition_inventory": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json",
            "sha256": sha256(disposition_bytes),
            "record_count": len(disposition["records"]),
            "local_path_exists": (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json").is_file(),
            "integration_status": "integrated" if (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json").is_file() else "pending_integration",
        },
        "source_removal_record": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-removal.json",
            "sha256": sha256(removal_bytes),
            "accepted_source_commit": FOUNDATION_OUTPUT,
            "removed_source_path_count": len(removal.get("removed_source_paths", [])),
            "local_path_exists": (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-removal.json").is_file(),
            "integration_status": "integrated" if (args.repo / "docs/implementation/pr38-final/results/t37-foundations/source-removal.json").is_file() else "pending_integration",
        },
        "maintained_documents": {
            path: sha256((args.repo / path).read_bytes())
            for path in (
                "docs/contracts/authoring.md",
                "docs/contracts/foundation-support.md",
                "docs/contracts/theming.md",
                "docs/contracts/migration.md",
                "docs/development.md",
                "crates/termrock-e2e/README.md",
                "crates/termrock-xtask/RULES.md",
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
            "id": "T37-CURSOR-CONTRACT-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-session/src/session.rs:33-56,300-351` and `crates/termrock-runtime/src/runtime.rs:3075-3082`",
            "disposition": "Auto is capability-gated at startup. Explicit Terminal, Block, Underline, and Bar choices select startup commands regardless of support; Bar selects SteadyBar. Runtime cursor requests are separate. Do not infer support or emitted output.",
            "maintained_destination": "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules",
            "record_indices": [],
        }, {
            "id": "T37-BACKDROP-CONTRACT-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-runtime/src/hit.rs:221-228,1372-1404` and `crates/termrock-runtime/src/capture.rs:147-172`",
            "disposition": "Document pass_through as stored hit metadata; no production routing reader was found. Preserve T15-PASS-THROUGH-01 as an open intended requirement. SecondaryTracker routes from SecondaryOutside independently. No routing failure or behavior was executed.",
            "maintained_destination": "docs/contracts/foundation-runtime.md#hit-ownership-and-releases",
            "record_indices": [],
        }, {
            "id": "T37-KEY-OBSERVE-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-core/src/keys.rs:285-306`",
            "disposition": "Observe learns Super unless explicitly refused. It learns Alt only when ALT is unset and protocol flags are active or the key code is outside Char, Esc, Enter, Backspace, Tab, and BackTab. Final hint delivery separately uses cached Alt policy and explicit refusal. No terminal behavior was proved.",
            "maintained_destination": "docs/contracts/foundation-values.md#keyboard-protocol-and-delivery",
            "record_indices": [],
        }, {
            "id": "T37-DIAGNOSTICS-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-core/src/diagnostics.rs:95-137`",
            "disposition": "Keep at most 64 retained items in this collection. Append preserves existing items before incoming retained items, then saturating-adds the prior dropped count. Dropped is cumulative usize saturation, not capped at 64; exact totals are unavailable beyond usize::MAX. Documentation correction only.",
            "maintained_destination": "docs/contracts/foundation-values.md#intent-claims-and-diagnostics",
            "record_indices": [],
        }, {
            "id": "T37-SELECTION-CONTRACT-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-core/src/selection.rs:264-337,280-303`",
            "disposition": "Range extension rebuilds from the saved base and adds selectable rows in the current range. Disabled rows in that base can remain; range-only rows from an earlier update can drop. select_only on a disabled row clears; toggle on a disabled row does nothing. Preserve T14 eligibility-transition policy and verification as open; no source defect or behavior proof is claimed.",
            "maintained_destination": "docs/contracts/foundation-values.md#list-selection-and-type-ahead",
            "record_indices": [],
        }, {
            "id": "T37-CURSOR-NEW-01",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-core/src/list.rs:132-140`",
            "disposition": "The constructor stores the supplied index with wrap disabled, vertical axis, and disabled-row skipping. It has no source length to clamp or validate. Empty or ignored navigation can retain the old index; settling nonempty source-dependent operations perform bounds work. No constructor defect is established.",
            "maintained_destination": "docs/contracts/foundation-values.md#list-selection-and-type-ahead",
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
        }, {
            "id": "T37-SUPPORT-R1",
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-shell/src/router.rs:382-446`",
            "disposition": "Document the modified-quit exception after an unconsumed layer response and keep the mismatch between source-comment and current keys-tier order open as T19-KEYS-TIER-01. No source change or behavior proof is authorized.",
            "maintained_destination": "docs/contracts/foundation-support.md#routing-order-open-contract",
            "record_indices": sorted(KEYS_TIER_INDICES),
        }, {
            "id": MODAL_DIM_DECISION,
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "source": "c817 `crates/termrock-theme/src/lib.rs:564-579`; immutable historical output difference",
            "disposition": "Keep Showcase/TablePro modal dim divergence open under T11-MODAL-DIM-01. T11 owns shared mapping policy; T22 and T29 own the respective consumer decisions. No visual acceptance or behavior change is claimed.",
            "maintained_destination": "docs/contracts/foundation-support.md#modal-backdrop-and-color-downgrade",
            "record_indices": [322],
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
        }, {
            "id": "T19-KEYS-TIER-01",
            "task": "T19",
            "status": "OPEN",
            "primary_owner": "T19 shell key routing owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "The source comments describe the keys tier as answering after page input. At c817, normal routing checks keys before chrome and page; capture routing gives the page a chance before keys. An unconsumed layer response ends ordinary routing, but an enabled modified quit chord may request quit before return.",
            "source_record_indices": [1005, 1008],
            "required_outcome": "Resolve whether the keys tier precedes or follows page input, while preserving the actual layer quit exception and explicit capture path.",
            "future_case_scope": ["capture enabled and disabled", "same chord in keys and page tiers", "consumed and ignored layer/menu responses", "selection and Ctrl+C copy", "exact modifier quit chords"],
            "limits": "No behavior case or source change is authorized in T37. Current behavior remains NOT VERIFIED.",
        }, {
            "id": "T11-MODAL-DIM-01",
            "task": "T11",
            "status": "OPEN",
            "primary_owner": "T11 shared modal backdrop mapping owner",
            "later_owners": ["T01 inventory", "T22 Showcase consumer owner", "T29 TablePro consumer owner", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "The frozen Showcase dialog underlays have no DIM runs. Frozen TablePro picker underlays have DIM at ANSI16 and color none. The shared mapping retains the TablePro-pinned DIM behavior. The source decision preserves this divergence; no new visual output was produced.",
            "source_record_indices": [322],
            "required_outcome": "T11 resolves the shared mapping policy with both Showcase and TablePro consumer requirements represented.",
            "future_case_scope": ["dialog and picker underlays", "ANSI16, Mono, and color planes", "blank and nonblank source cells", "nested and disabled modal layers", "glyph, modifiers, and DIM output"],
            "limits": "T37 does not change visual behavior or claim visual acceptance. Product behavior remains NOT VERIFIED.",
        }, {
            "id": "T15-PASS-THROUGH-01",
            "task": "T15",
            "status": "OPEN",
            "primary_owner": "T15 runtime hit and secondary routing owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "At c817, Registry stores and compares pass_through metadata, but no production routing reader was found. Deleted tests were the only getter callers. SecondaryTracker routes by independent SecondaryOutside and returns its own result.",
            "required_outcome": "Implement or explicitly migrate the intended public pass-through behavior. Extend the bounded hit/capture source ownership only if required by the approved design.",
            "future_case_scope": ["primary and secondary hit pass-through true/false", "Ignore, Close, Reopen, and nested modal barriers", "disabled barriers", "exactly-once routing", "current-frame and presentation ownership"],
            "limits": "T37 changes no source and executes no routing failure. Behavior remains NOT VERIFIED.",
        }, {
            "id": "T14",
            "task": "T14",
            "status": "OPEN",
            "primary_owner": "T14 list selection owner",
            "later_owners": ["T01 inventory", "T38 reviewed public-path cases", "T36 final closure"],
            "source_fact": "Selection operations accept current enabled-row predicates but do not globally sanitize old selection when eligibility changes. Range extension can preserve disabled rows in its saved base and drop range-only disabled rows from an earlier update.",
            "required_outcome": "Define consumer policy and verification for eligibility transitions without treating the current behavior as a source defect or adding a replacement test in T37.",
            "future_case_scope": ["disabled row already in saved base", "row disabled after selection", "range-only row disabled before next extension", "select_only and toggle on disabled rows", "Select All and activation eligibility"],
            "limits": "T37 records current source semantics only. Product behavior remains NOT VERIFIED.",
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
        "attachment_audit": {
            "reviewer": "/root/t00_independent_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "fixed_source_commit": SOURCE_INPUT,
            "selected_groups": 496,
            "retired_or_test_only_groups": 55,
            "surviving_module_groups": 14,
            "surviving_non_module_groups": 427,
            "same_scope_matches": 422,
            "recipe_scope_corrections": [
                {"source_map_record": 325, "attached_item": "enum declaration `ManagerPresentationProfile`", "scope": "module root"},
                {"source_map_record": 328, "attached_item": "variant `Paper`", "scope": "enum `ManagerPresentationProfile`"},
                {"source_map_record": 329, "attached_item": "struct declaration `AuthoredDeclaration`", "scope": "module root"},
                {"source_map_record": 330, "attached_item": "method `glyph_slot`", "scope": "impl `PartEdit<'_>`"},
                {"source_map_record": 331, "attached_item": "method `size_slot`", "scope": "impl `PartEdit<'_>`"}
            ],
            "method": "Strip comments and literals into Rust tokens, track a lexical brace stack, and require the attached declaration signature in the same container scope in the clean tree. This is structural evidence, not a Rust parser or semantic proof.",
            "status": "CORRECTION_APPLIED_PENDING_INDEPENDENT_REVIEW"
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
