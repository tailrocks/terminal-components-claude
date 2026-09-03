# Adjudication K — `Form` API and the `Grid::update` bound

**Status:** proposed. Resolves the two items left open at `COMPONENT_ARCHITECTURE.md:3677-3681` ("Not applied — requires a fresh `opus-analyst` decision"). Nothing here reopens Adjudications A–J.

**Convention:** **[F]** = collected fact with a citation. Everything else is decision or inference.

---

## 0. Facts the two decisions rest on

### 0.1 The three hand-built form engines

| # | Engine | Location | Shape |
|---|---|---|---|
| 1 | jackin `FormDialog` | `src/bin/jackin_preview/screens/modals.rs:787-1541` | Data-declared fields (`FormField{name,kind,visible}`, `:812-816`), 5 field kinds (`FieldKindW`, `:796-810`), one key router, one click router, scroll, action row |
| 2 | TablePro `ConnForm` | `src/bin/tablepro/connections.rs:62-87`, routers `:573-687`, `:793-861`, `:863-887`, layout `:1120-1256` | 17 controls + a `Tabs` section strip + 4 buttons, three hand-written `if f == …` ladders, manual height arithmetic |
| 3 | TablePro `FilterEditor` | `src/bin/tablepro/app.rs:99-109`, built `:1368-1433` | 4 controls + 2 buttons, rebuilt wholesale on every open |

**[F]** DOM §4.1 J2 (`docs/audit/domain-boundary-audit.md:394`) records these as "three independent form engines in one repo" and names the exact required capabilities: *ordered fields, visibility toggling, scroll with focused-field reveal, action buttons, error row, nested popup*. **[F]** §14.2 J2 (`COMPONENT_ARCHITECTURE.md:1208`) already adjudicated the disposition — **library component `Form` + `Field<C>`** — and left only the API sketch open. **[F]** Appendix A assigns `components/form.rs` to work package **4F**, depending on **4B** (`Field`), wave 2 (`COMPONENT_ARCHITECTURE.md:3112`, `:3119`).

### 0.2 Capability inventory (each must survive)

| Capability | Evidence |
|---|---|
| Ordered, data-declared fields | `modals.rs:812-816`, `:937-954` |
| Heterogeneous kinds: input, select, checkbox, radio, chooser, note | `modals.rs:796-810`; TablePro adds `TextArea` (`connections.rs:81`, `:198-204`), `Toggle` (`:77-78`, `:82`) |
| Secret field | **[F]** TablePro's password is a *plain* `TextInput` with a placeholder, **not** `.masked()` (`connections.rs:155-157`) — a live defect |
| Conditional visibility | `modals.rs:815`, `:866-869`, `:984-988`; TablePro does it as `disabled` set **inside render** (`connections.rs:1205-1206`) |
| Sections / tabs | `connections.rs:134-135`, `:1145`, `:1197`; tab error flag set **inside render** (`connections.rs:1134`) |
| Two-column layout, half-width pairs | `connections.rs:1143` (58/30/24 split), host+port pair `:1153-1156` |
| Automatic Tab order | `modals.rs:1084-1091` (manual `focus.next/prev`); `connections.rs:587-588` |
| Scroll-to-focused-field | `modals.rs:1356-1371` — **mutates `ScrollState` from `render`** |
| Per-field validation | `input.rs` validator fn-pointer; TablePro free fns `connections.rs:89-105` |
| Form-level validation + focus-to-first-error | `connections.rs:246-250` (`a && b`), `:704-711` (manual focus pick) |
| Dirty tracking | `modals.rs:930`, set at `:1110`, `:1121`, `:1142`, `:1153`, `:1248` |
| Action row (submit/cancel/extra) | `modals.rs:921-924`, `:1480-1489`; TablePro has four (`connections.rs:83-86`, `:1232-1242`) |
| Enter-submits policy | `modals.rs:1204-1212` (`Enter if !editing`); TablePro uses `Ctrl+S` (`connections.rs:577`) |
| Left/Right traversal of the action row | `modals.rs:1213-1230` |
| Nested popover (open `Select` inside the form) | `modals.rs:1068-1072` (guard), `:1374`, `:1393-1399`, `:1514-1539` ("an open select popup draws last") |
| Nesting inside a dialog layer | `modal_frame` at `modals.rs:1340-1349` |
| Chooser field (button + value + detail) | `modals.rs:801-807`, `:1018-1029`, render `:1402-1436` |
| Note / help rows | `modals.rs:808-810`, `:1031-1037`, render `:1437-1448` |
| Cross-field effect (engine → default port) | `connections.rs:621-634` — **rebuilds the `TextInput`** to change its value |

### 0.3 Constraints this design must honour

* **Props built once, never from `&self`** — `COMPONENT_ARCHITECTURE.md:1135`, `:3651`; enforced by `architecture::props_are_built_once`. The rule names `Form::field(id, …)` explicitly as the mechanism for a 15-field form.
* **Data passed to phase calls, never held in props** — §21 item 1, `:3393-3411`; §13 table row `:1112`; §17.0 A3 `:1855-1864`.
* **`FieldControl` is draw-time chrome only; `Field` has no `Id`** — §21 item 7, `:3479-3497`; §15 `:1274-1294`.
* **`Response<A>`, one action per response; `BitOr` only for `Response<()>`** — §21 item 4, `:3432-3439`; §6.1 `:413-417`.
* **Controlled `&mut` values** — §4 rule 4 `:303`; S4 `:313`.
* **`draw` is `&self` + `&XState`** — §3.1 `:120`, `:122`; R2 `:323`.
* **Containers register `Decorative` regions** — §21 item 13, `:3545-3556` (which already names `Form` as a container).
* **Esc reaches the focused editor before the layer** — §21 item 3, `:3426-3428`.

---

## K1 — the `Form` API

### K1.1 Decision

**`Form` is a library component (`components/form.rs`, work package 4F), not a `FormState` + `layout::form` helper pair.**

Reasons, in decreasing weight:

1. **Three of the required capabilities are `update`-phase semantics, which a layout helper cannot own.** Enter-submits arbitration against the focused control's `EDITING`/`swallows_typing` flag; the validate-then-focus-first-error sequence; and the commit of the in-flight edit before validation. A `layout::form` helper runs in `draw`, where `&self`/`&XState` make all three a compile error (`:122`).
2. **Scroll-to-focused-field is today a render-time mutation** (`modals.rs:1356-1371`). Under the accepted model the only legal home for it is `update`, which needs the field order, the field heights and the focus — i.e. exactly the component's own knowledge. A helper would push it back to every screen.
3. **`architecture::conformance_covers_every_public_component`** (`:1636`) makes registration mandatory for any public component, and `FormCase` is *already* in the `conformance_suite!` list (`:1440`). A helper pair would have to be removed from that list, weakening `draw_does_not_commit_or_cancel` and `secret_never_appears_in_debug` for the highest-risk surface in the repository.
4. **§14.2 J2 (`:1208`) already decided "Library component — `Form`".** Re-deciding it as a helper reopens an accepted adjudication for no invariant.

`Form` is a **composition** (disposition C), not an overlay: it opens no layer, traps no focus, paints no frame. It is placed *inside* a `Dialog` body slot, a `Panel`, or a bare rect.

### K1.2 Exact Rust API

```rust
// ───────────────────────────── identity ─────────────────────────────
// A field is addressed by its own `Id`. There is NO new key type: §7.1's `Id` is
// Copy + Eq + Hash, and "matching a screen's own const Ids" is the one form of
// manual dispatch §14.1 (:1221) leaves in application code.

// ───────────────────────────── declaration ──────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum FieldSpan { Full, Half }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct GroupKey(u16);
impl GroupKey { pub const ALL: GroupKey; pub const fn custom(name: &'static str) -> GroupKey; }

/// Configuration only — no values, no `&mut`, no `Id`-less chrome. Every variant is a
/// props struct built by the same consuming builders it has standalone (§13).
pub enum FieldKind<'a> {
    Text   (TextInput<'a>),          // `.secret(SecretPolicy)` already applied for secrets
    Area   (TextArea<'a>),
    Select (Select<'a>),
    Radio  (RadioGroup<'a>),
    Check  (Checkbox<'a>),
    Toggle (Toggle<'a>),
    Chips  (ChipBar<'a>),
    Chooser(Button<'a>),             // button + a read-only value line + optional detail
    Note,                            // static rows; no focus stop, Decorative regions only
}

pub struct FieldSpec<'a> {
    pub id:       Id,
    pub label:    &'a str,
    pub kind:     FieldKind<'a>,
    pub required: bool,
    pub help:     Option<&'a str>,
    pub span:     FieldSpan,         // Full | Half — the two-column form
    pub group:    GroupKey,          // section/tab membership; GroupKey::ALL is always shown
    pub plain:    bool,              // forwarded to Field::plain (kills `TextInput::plain_label`)
}
impl<'a> FieldSpec<'a> {
    pub const fn new(id: Id, label: &'a str, kind: FieldKind<'a>) -> Self;
    pub const fn required(self, yes: bool) -> Self;
    pub const fn help(self, s: &'a str) -> Self;
    pub const fn span(self, s: FieldSpan) -> Self;
    pub const fn group(self, g: GroupKey) -> Self;
    pub const fn plain(self, yes: bool) -> Self;
}

// ───────────────────────────── the data channel ─────────────────────
// Values are the CALLER's. They are passed to each phase call, exactly like `Grid`'s
// model (§21 item 1) and `List`'s items. `Form<'a>` never holds a value or a `&mut`.
pub enum FieldMut<'d> {
    Text  (&'d mut String),
    Secret(&'d mut Secret),
    Choice(&'d mut usize),
    Flag  (&'d mut bool),
    Chips (&'d mut KeySet),
    ReadOnly,                        // Chooser / Note: no controlled value
}
pub enum FieldRef<'d> {
    Text   (&'d str),
    Secret (&'d Secret),             // masked by Secret::write_mask; never stringified
    Choice (usize),
    Flag   (bool),
    Chips  (&'d KeySet),
    Display{ value: &'d str, detail: Option<&'d str> },   // Chooser
    Note   (&'d [(&'d str, Role)]),
}

pub trait FormData {
    fn value    (&self,     id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;
    fn visible  (&self, _id: Id) -> bool { true }
    fn disabled (&self, _id: Id) -> bool { false }
    /// External / async / server-side errors. Per-field local errors live in `FormState`.
    fn error    (&self, _id: Id) -> Option<&str> { None }
    fn validate (&self, _id: Id, _v: FieldRef<'_>) -> Result<(), FieldError> { Ok(()) }
    /// Cross-field rules. Runs after every per-field check passes.
    fn validate_all(&self) -> Result<(), (Id, FieldError)> { Ok(()) }
}

// ───────────────────────────── the component ────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterPolicy { SubmitsWhenIdle, Never }   // typed enum, not a bool (§13 "no boolean soup")

pub struct Form<'a> { /* id, fields, actions, submit, cancel, enter, columns, group */ }

impl<'a> Form<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::BODY, Part::ACTIONS,
                                         Part::HELP, Part::MARKER, Part::TRACK, Part::THUMB];
    pub fn new(id: Id, fields: &'a [FieldSpec<'a>]) -> Self;
    pub fn actions(self, a: &'a [Action<'a>]) -> Self;    // §17.0 A4; `.chord()` gives Ctrl+S
    pub fn submit(self, k: ActionKey) -> Self;            // default ActionKey::SAVE
    pub fn cancel(self, k: ActionKey) -> Self;            // default ActionKey::CANCEL
    pub fn enter(self, p: EnterPolicy) -> Self;           // default SubmitsWhenIdle
    pub fn columns(self, n: u8) -> Self;                  // default 1
    pub fn group(self, g: GroupKey) -> Self;              // active section; default GroupKey::ALL
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;

    pub fn update<D: FormData + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut FormState, data: &mut D)
        -> Response<FormAction>;
    pub fn draw<D: FormData + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &FormState, data: &D)
        -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormAction {
    Changed  (Id),          // a control reported a value change (draft level)
    Committed(Id),          // a control committed; validation for that field has run
    Chose    (Id),          // a Chooser button fired — the owner opens its own picker
    Action   (ActionKey),   // any action-row button, INCLUDING submit once it validates
    Invalid  (Id),          // submit refused; `Id` is the first invalid field, already focused
}
// FormAction carries no value of any kind. This is the secret-containment invariant (F5).

pub struct FormState { /* slots: SmallVec<[FieldSlot; 16]> keyed by Id, scroll: ScrollState,
                          errors: SmallVec<[(Id, FieldError); 4]>, dirty: bool, gen stamp */ }
impl FormState {
    pub fn is_dirty(&self) -> bool;
    pub fn mark_clean(&mut self);
    pub fn error(&self, id: Id) -> Option<&FieldError>;
    pub fn set_error(&mut self, id: Id, e: Option<FieldError>);
    pub fn clear_errors(&mut self);
    pub fn reveal(&mut self, id: Id);          // request scroll-to-field on the next layout
    pub fn zeroize(&mut self);                 // overwrites every secret draft (§15)
}
impl Default for FormState { /* empty; slots are created by the first `update` */ }
impl Reconcile for FormState { /* slots follow the declared field ids, §21 item 21 */ }
impl fmt::Debug for FormState { /* manual; every draft renders as "[redacted]" */ }
```

### K1.3 Invariants

* **F1 — Props hold no data.** `Form<'a>` holds `&'a [FieldSpec<'a>]`, `&'a [Action<'a>]` and scalars. It never holds a `String`, a `&mut`, or `&self` of a screen. One private constructor per screen returns the field array; both phases call it (`architecture::props_are_built_once`, `:1642`).
* **F2 — Tab order is declaration order.** `Form::draw` registers each visible, focusable field's control in `fields` order; §8.1's *traversal order is registration order* (`:1453`) supplies Tab and Shift+Tab. `Form` never calls `cx.focus` for traversal. Deletes `modals.rs:1084-1091` and `connections.rs:587-588`.
* **F3 — Hidden fields register nothing and keep their drafts.** `FormData::visible(id) == false` ⇒ no ring entry, no region, no measure contribution; the `FieldSlot` survives in `FormState` keyed by `Id`, so toggling back restores the draft and the cursor. Replaces `modals.rs:815/866-869/984-988` and the render-time `disabled` write at `connections.rs:1205-1206`.
* **F4 — Field height is a pure function of `(FieldSpec, DesignTokens, width)`.** Both phases compute it identically; `update` reaches the tokens through `FrameRead::design()` on `Cx` (`:1783-1788`), and the body width through `cx.area(self.id)` (last frame; `None` on frame 1 is a documented no-op, S3 `:312`). This is what lets `update` own scroll-to-focused-field without `Ui`. Deletes the `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic at `connections.rs:1144-1186` and `modals.rs:871-880`.
* **F5 — No value ever leaves the form.** There is no `values()`. `FormAction` carries only `Id` and `ActionKey`. See §K1.5.
* **F6 — `draw` commits nothing.** `Form::draw` is `&self` + `&FormState`; the blur commit is an `Intent::FocusOut`-driven transition in `update` with the control's `BlurPolicy` (§15 `:1242-1250`). Covered by `conformance::form::draw_does_not_commit_or_cancel` (case 7, `:1457`).
* **F7 — At most one `FormAction` per frame.** `Response<A>` holds `Option<A>` (`:382`) and `BitOr` is defined only for `Response<()>` (§21 item 4). `Form::update` folds each control's `flow`/`invalidate` and keeps the **first** action in declaration order, ordering action-row buttons last; the rest are recorded as `invalidate` only. This is the same rule the ladder implements today by `return`ing on the first hit (`modals.rs:1112`, `:1126`).
* **F8 — Nested popovers are layers, not draw order.** An open `Select` opens a `Popover` via `cx.open_layer` in its own `update`; its content is painted inside `ui.layer` from `Select::draw`. Z-order is the `LayerId` assigned at open, not the call order (§21 item 14, `:3558`). `Form` needs no `any_open_select()` guard (`modals.rs:1068-1072`), no deferred re-render (`modals.rs:1514-1539`), and no manual hit re-registration (`modals.rs:1491-1512`). Esc reaches the `Select` before the layer and before the enclosing dialog (§21 item 3).
* **F9 — `Form` is layer-agnostic.** Inside a `Dialog`, the trap, backdrop, Esc and focus restore belong to the layer (§9.1). `Form` registers its container and section chrome as `Decorative` (§21 item 13, which already names `Form`), so a click on empty form background is an ordinary miss and never records `UndeliveredIntent`.
* **F10 — Submit sequence is fixed and total.** On the submit `ActionKey`: (1) blur-commit the focused control if editing; (2) for every **visible** field in declaration order, `FormData::validate(id, value)`; (3) `FormData::validate_all()`; (4) on the first failure — `st.set_error`, `st.reveal(id)`, `cx.focus(id)`, emit `FormAction::Invalid(id)`; (5) otherwise emit `FormAction::Action(submit)`. Replaces `connections.rs:246-250` + `:704-711` and gives jackin's `FormDialog` the validation it has none of (**[F]** DOM §3.2, `docs/audit/domain-boundary-audit.md:354`).
* **F11 — Enter-submits is arbitrated, not guessed.** `EnterPolicy::SubmitsWhenIdle` submits only when the focused control's focus entry does not set `swallows_typing` and does not carry `StateFlags::EDITING`. Generalises `modals.rs:1204-1212`'s `if !editing` and the six ad-hoc `!editing` guards §13.1 names (`:1174`). A submit chord (`Ctrl+S`, `connections.rs:577`) is declared as `Action::new(SAVE, "Save").chord(Chord::with(KeyCode::Char('s'), CTRL))` — no new API (§17.0 A4, `:1879`).
* **F12 — Dirty is set only by a committed change.** `FormAction::Committed` and a toggle/choice change set `st.dirty`; a keystroke inside a draft does not. This corrects `modals.rs:1119-1121`, where a mid-edit `InputEvent::Changed` sets `dirty` before anything is committed.
* **F13 — `FormState` redacts.** Manual `Debug` on `FormState` and every `FieldSlot`; `zeroize()` overwrites secret drafts before drop (§15 `:1297`).

### K1.4 Example-11-style usage — the 15-field connection form (condensed)

```rust
use junie_tui::{id, Action, ActionKey, Button, Checkbox, Cx, EnterPolicy, FieldKind, FieldRef,
                FieldMut, FieldSpan, FieldSpec, Form, FormAction, FormData, FormState, GroupKey,
                Id, KeyCode, KeyModifiers, Chord, RadioGroup, Response, Secret, SecretPolicy,
                Select, TextArea, TextInput, Toggle, Ui, Rect, FieldError};

const FORM: Id = id!("connections.form");
const NAME:  Id = id!("connections.form.name");     const ENGINE: Id = id!("connections.form.engine");
const HOST:  Id = id!("connections.form.host");     const PORT:   Id = id!("connections.form.port");
const DB:    Id = id!("connections.form.db");       const USER:   Id = id!("connections.form.user");
const PW:    Id = id!("connections.form.pw");       const ASKPW:  Id = id!("connections.form.askpw");
const ENV:   Id = id!("connections.form.env");      const GROUP:  Id = id!("connections.form.group");
const SAFE:  Id = id!("connections.form.safe");     const SSL:    Id = id!("connections.form.ssl");
const SSH:   Id = id!("connections.form.ssh");      const SSHH:   Id = id!("connections.form.sshhost");
const START: Id = id!("connections.form.startup");
const BASIC: GroupKey = GroupKey::custom("basic");  const ADV: GroupKey = GroupKey::custom("adv");
const K_TEST: ActionKey = ActionKey::custom("test");
const K_SAVE_CONNECT: ActionKey = ActionKey::custom("save+connect");

/// The 15 declarations, written ONCE, called from both phases (§13 props-built-once).
/// Takes only what it needs — never `&self` — so `update` keeps `&mut self.values`.
fn conn_fields<'a>(engines: &'a [&'a str], envs: &'a [&'a str],
                   groups: &'a [&'a str], modes: &'a [&'a str]) -> [FieldSpec<'a>; 15] {
    use FieldKind::*;
    [
        FieldSpec::new(NAME,  "Name",     Text(TextInput::new(NAME))).required(true).group(BASIC),
        FieldSpec::new(ENGINE,"Engine",   Select(Select::new(ENGINE, engines))).group(BASIC),
        FieldSpec::new(HOST,  "Host",     Text(TextInput::new(HOST).placeholder("localhost")))
            .help("Blank: driver default").span(FieldSpan::Half).group(BASIC),
        FieldSpec::new(PORT,  "Port",     Text(TextInput::new(PORT).validate(&port_rule)))
            .span(FieldSpan::Half).group(BASIC),
        FieldSpec::new(DB,    "Database", Text(TextInput::new(DB)))
            .help("Required for PostgreSQL").group(BASIC),
        FieldSpec::new(USER,  "Username", Text(TextInput::new(USER))).group(BASIC),
        FieldSpec::new(PW,    "Password", Text(TextInput::new(PW).secret(SecretPolicy::default())))
            .help("Never written to connections.json").group(BASIC),
        FieldSpec::new(ASKPW, "",         Check(Checkbox::new(ASKPW, "Prompt for password on connect")))
            .plain(true).group(BASIC),
        FieldSpec::new(ENV,   "Environment", Radio(RadioGroup::new(ENV, envs))).group(BASIC),
        FieldSpec::new(GROUP, "Group",    Select(Select::new(GROUP, groups))).group(BASIC),
        FieldSpec::new(SAFE,  "Safe Mode",Radio(RadioGroup::new(SAFE, modes))).group(BASIC),
        FieldSpec::new(SSL,   "",         Toggle(Toggle::new(SSL, "Use SSL / TLS"))).plain(true).group(ADV),
        FieldSpec::new(SSH,   "",         Toggle(Toggle::new(SSH, "SSH tunnel"))).plain(true).group(ADV),
        FieldSpec::new(SSHH,  "SSH host", Text(TextInput::new(SSHH).placeholder("bastion.example.com")))
            .group(ADV),
        FieldSpec::new(START, "Startup commands", Area(TextArea::new(START, 3)))
            .help("Run after every connect, one per line").group(ADV),
    ]
}

fn conn_actions() -> [Action<'static>; 4] {
    [Action::quiet(K_TEST, "Test connection"),
     Action::new(ActionKey::CANCEL, "Cancel"),
     Action::new(ActionKey::SAVE, "Save").chord(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)),
     Action::new(K_SAVE_CONNECT, "Save & connect")]
}

/// The caller owns every value. This struct IS the connection draft.
#[derive(Default)]
struct Draft { name: String, engine: usize, host: String, port: String, db: String,
               user: String, pw: Secret, ask_pw: bool, env: usize, group: usize, safe: usize,
               ssl: bool, ssh: bool, ssh_host: String, startup: String }

impl FormData for Draft {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            NAME => FieldRef::Text(&self.name),        ENGINE => FieldRef::Choice(self.engine),
            HOST => FieldRef::Text(&self.host),        PORT   => FieldRef::Text(&self.port),
            DB   => FieldRef::Text(&self.db),          USER   => FieldRef::Text(&self.user),
            PW   => FieldRef::Secret(&self.pw),        ASKPW  => FieldRef::Flag(self.ask_pw),
            ENV  => FieldRef::Choice(self.env),        GROUP  => FieldRef::Choice(self.group),
            SAFE => FieldRef::Choice(self.safe),       SSL    => FieldRef::Flag(self.ssl),
            SSH  => FieldRef::Flag(self.ssh),          SSHH   => FieldRef::Text(&self.ssh_host),
            START=> FieldRef::Text(&self.startup),     _      => FieldRef::Text(""),
        }
    }
    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            NAME => FieldMut::Text(&mut self.name),    ENGINE => FieldMut::Choice(&mut self.engine),
            HOST => FieldMut::Text(&mut self.host),    PORT   => FieldMut::Text(&mut self.port),
            DB   => FieldMut::Text(&mut self.db),      USER   => FieldMut::Text(&mut self.user),
            PW   => FieldMut::Secret(&mut self.pw),    ASKPW  => FieldMut::Flag(&mut self.ask_pw),
            ENV  => FieldMut::Choice(&mut self.env),   GROUP  => FieldMut::Choice(&mut self.group),
            SAFE => FieldMut::Choice(&mut self.safe),  SSL    => FieldMut::Flag(&mut self.ssl),
            SSH  => FieldMut::Flag(&mut self.ssh),     SSHH   => FieldMut::Text(&mut self.ssh_host),
            START=> FieldMut::Text(&mut self.startup), _      => FieldMut::ReadOnly,
        }
    }
    // conditional visibility — replaces the render-time `disabled` write at connections.rs:1205-1206
    fn visible(&self, id: Id) -> bool { if id == SSHH { self.ssh } else { true } }
    fn validate(&self, id: Id, v: FieldRef<'_>) -> Result<(), FieldError> {
        match (id, v) {
            (NAME, FieldRef::Text(s)) if s.trim().is_empty() =>
                Err(FieldError { message: "Required".into(), code: Some("required") }),
            (PORT, FieldRef::Text(s)) => port_rule(s),
            _ => Ok(()),
        }
    }
    fn validate_all(&self) -> Result<(), (Id, FieldError)> {
        if self.engine == 0 && self.db.trim().is_empty() {
            return Err((DB, FieldError { message: "PostgreSQL needs a database".into(), code: None }));
        }
        Ok(())
    }
}

struct ConnScreen { draft: Draft, form: FormState, tab: GroupKey, /* … */ }

impl ConnScreen {
    fn form_props<'a>(&self, f: &'a [FieldSpec<'a>], a: &'a [Action<'a>]) -> Form<'a> {
        Form::new(FORM, f).actions(a).submit(ActionKey::SAVE).cancel(ActionKey::CANCEL)
            .enter(EnterPolicy::SubmitsWhenIdle).columns(2).group(self.tab)
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let (fields, actions) = (conn_fields(ENGINES, ENVS, GROUPS, MODES), conn_actions());
        self.form_props(&fields, &actions)
            .update(cx, &mut self.form, &mut self.draft)          // data per phase (§21 item 1)
            .on_action(|a| match a {
                // the ONE cross-field rule; no widget is rebuilt (cf. connections.rs:629-631)
                FormAction::Committed(ENGINE) =>
                    self.draft.port = default_port(self.draft.engine).to_owned(),
                FormAction::Action(ActionKey::SAVE)   => self.save(false, cx),
                FormAction::Action(K_SAVE_CONNECT)    => self.save(true,  cx),
                FormAction::Action(K_TEST)            => self.begin_test(),
                FormAction::Action(ActionKey::CANCEL) => self.close(cx),
                FormAction::Invalid(_)                => cx.record("form.invalid"),
                _ => {}
            })
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let (fields, actions) = (conn_fields(ENGINES, ENVS, GROUPS, MODES), conn_actions());
        self.form_props(&fields, &actions).draw(ui, area, &self.form, &self.draft);
    }
}
```

**What disappears:** `on_form_key` (`connections.rs:573-687`, 115 lines with the `input!` macro), `on_form_click` (`:793-861`), `on_paste` (`:863-887`), `render_form`'s height arithmetic (`:1144-1228`), `validate()` (`:246-250`), the focus-to-first-error block (`:704-711`), the widget rebuild at `:629-631`, and the render-time writes at `:1134` and `:1205-1206`. `ConnForm` (`:62-87`) becomes `Draft` — 15 owned values, which §20.7 (`:2969`) already accepted as the cost of controlled values.

### K1.5 How `values()` reaches the app without cloning secrets — exactly

**There is no `values()`.** The mechanism is elimination, not redaction.

* **[F]** Today `FormDialog::values()` (`modals.rs:1039-1044`) clones **every** field name and value into `FormValues = Vec<(String, FieldValue)>` (`:794`), including `FieldKindW::Input(TextInput)` password fields; **[F]** DOM §3.3 (`docs/audit/domain-boundary-audit.md:364`) traces that vector through `ModalResult::Form(Some(values))` into `accounts.rs`'s `CredentialSource` handling. Every API key in jackin is copied at least twice per submit.
* Under this design the values were never inside the form. `Draft` is an ordinary field of the screen. `Form::update` reaches each value through `FormData::value_mut(id)` for the duration of one control's `update` call and writes through it; `Form::draw` reaches it through `FormData::value(id)` and reads it. Both borrows end when the phase method returns.
* On submit the screen already owns the values — `self.draft` — so the "result" crosses no boundary. `FormAction::Action(ActionKey::SAVE)` is a 2-word `Copy` enum.
* Secrets specifically: `FieldMut::Secret(&mut Secret)` / `FieldRef::Secret(&Secret)`. `Secret` is **not `Clone`, not `PartialEq`, not `Serialize`** (§15, `:1270`), so a clone is a compile error, not a review item. `Form::draw` renders it with `Secret::write_mask(&mut CellUi, n)` (§21 item 30 P5, `:3645`), which writes cells with a **synthetic** tail — no `String` of the secret is ever constructed. The in-flight draft lives in the field's `TextInputState` inside `FormState`, whose `Debug` is manual and redacting (F13) and whose `zeroize()` overwrites it (§15 `:1297`).
* Closing the enclosing layer calls `FormState::zeroize()` on `LayerEvent::Dismissed`/`Closed`, so a cancelled password is overwritten rather than dropped.

### K1.6 Rejected alternatives

| Rejected | Why |
|---|---|
| **`Form` holds `&'a mut` per field** (a `FormFields<'a>` built fresh in `update`) | The props struct would be built differently in the two phases — the exact silent-bug class §13's props-built-once rule exists to kill (`:1135`), and `architecture::props_are_built_once` would flag it. It also re-creates the B3 borrow conflict §21 item 1 removed (`:3409`). |
| **`FormState` + `layout::form` helper pair** | Enter arbitration, validate-then-focus, blur-commit and scroll-to-field are `update`-phase semantics; a `draw` helper cannot own any of them (`:122`). Also drops `FormCase` out of `conformance_suite!` (`:1440`). |
| **Wide `FormData` with one accessor per type** (`text/text_mut/choice/set_choice/flag/…`) | 12+ methods, all defaulted, so a forgotten override compiles into a silently empty field. The two-accessor `FieldMut`/`FieldRef` shape makes the screen's `match` exhaustive over its own `const Id`s. |
| **A `FieldKey(u16)` newtype** | A second key space over the `Id` that already keys focus, hits, `cx.area`, `Response::for_id` and `Harness::area_of`. §7.1 already forbids parallel identity (`:533`). |
| **`FormData::on_change(id)` hook** | Redundant with `FormAction::Committed(id)`, and a `&mut self` callback re-entered from inside `update` re-opens draw-order-style implicit ordering. |
| **`values() -> FormValues` kept for convenience** | It is the secret-leak mechanism (§K1.5) and `Secret: !Clone` makes it uncompilable for secret fields anyway. |
| **`bool` parameters for enter/columns/sections** | §13 "no boolean parameter soup" (`:1131`); hence `EnterPolicy`, `FieldSpan`, `GroupKey`. |
| **`Form` opening its own layer** | Duplicates §9's stack; a form is used both inside a dialog (`modals.rs:1340`) and inline on a page (`connections.rs:1130-1132`). |

### K1.7 Acceptance conditions (executable)

```bash
cargo test -p junie-tui --lib form::                       # the unit block below
cargo test -p junie-tui --test conformance conformance::form::
cargo test -p junie-tui --test render   render::components::form::
cargo test -p junie-tui --test architecture architecture::props_are_built_once
cargo test -p tablepro -p jackin-preview
cargo test -p junie-tui --test perf --release -- --test-threads=1 frame_tablepro_connection_form
```

Named tests (added to §16.1 / §16.4 / §16.6 inventories, so `architecture::every_named_test_exists` covers them):

* `form::tab_order_follows_declaration_order_skipping_hidden`
* `form::hidden_field_registers_no_ring_entry_and_keeps_its_draft`
* `form::field_height_is_a_pure_function_of_spec_and_design_tokens`
* `form::scroll_reveals_the_focused_field_from_update_not_draw`
* `form::submit_commits_the_in_flight_edit_before_validating`
* `form::submit_validates_every_visible_field_then_focuses_the_first_error`
* `form::submit_skips_hidden_fields_during_validation`
* `form::enter_submits_only_when_the_focused_control_is_not_editing`
* `form::submit_chord_is_declared_on_the_action_not_baked_in`
* `form::dirty_is_set_by_a_commit_not_by_a_keystroke`
* `form::chooser_activation_emits_chose_with_the_field_id`
* `form::note_rows_register_only_decorative_regions`
* `form::at_most_one_action_per_frame_in_declaration_order`
* `form::open_select_popover_traps_focus_and_esc_closes_only_the_popover`
* `form::form_action_variants_carry_no_value` (exhaustive `match`, one arm per variant)
* `form::zeroize_overwrites_every_secret_draft`
* `conformance::form::draw_does_not_commit_or_cancel` *(Caps::EDITS — generated)*
* `conformance::form::secret_never_appears_in_debug` *(Caps::SECRET — generated)*
* `conformance::form::survives_tiny_rects_0x0_to_3x3`, `…::draw_twice_leaves_state_equal` *(generated)*
* `tablepro::connection_form_keyboard_and_mouse_reach_every_field`
* `tablepro::connection_form_focuses_the_first_invalid_field`
* `tablepro::connection_password_is_masked_and_absent_from_the_frame` *(closes the `connections.rs:155-157` defect)*
* `jackin::form_dialog_toggles_visibility_and_keeps_drafts`
* `jackin::form_dialog_secret_never_reaches_the_screen_as_a_string`
* perf `frame_tablepro_connection_form_120x40` — **< 40 allocs/frame**

Grep conditions:

```bash
! rg -n 'fn values\(|FormValues|FieldValue::' src/bin apps/           # the cloning channel is gone
! rg -n 'focus\.(next|prev|focus)\(' apps/tablepro/src/connections.rs apps/jackin-preview/src/screens/
! rg -n 'TextInput::HEIGHT|Select::HEIGHT|\.height\(\) *\+' apps/      # manual field arithmetic
```

### K1.8 Risks

1. **`FieldKind` is a closed enum.** A downstream control cannot be a form field. Mitigation: `FieldKind` is the *library* set; a custom control is composed beside the form, or `FieldKind::Custom(&'a dyn FieldControl<State = …>)` is added later — deliberately deferred, because the associated `State` type makes a uniform `dyn` slot non-trivial and no current consumer needs it.
2. **`FieldSlot` is a per-kind enum inside `FormState`**, so `FormState: Clone + PartialEq` (S2, `:311`) requires every control state to be. All are (`:1401`).
3. **First-frame scroll.** `cx.area(FORM)` is `None` on the first frame, so scroll-to-focused-field is a no-op then (S3). Visible only if a form opens already scrolled; documented, and one frame later it corrects.
4. **`FormData` match ladders are unchecked for coverage.** A missing arm falls to `_ => FieldRef::Text("")`. Mitigation: the `_` arm should be `unreachable_field(id)` in app code, and `form::every_declared_field_resolves_a_value` is a debug-assert-backed unit test the app suites run over each real `FormData`.
5. **`Secret` in `Draft` means `Draft: !Clone`.** TablePro's `ConnForm::to_connection(base)` currently clones (`connections.rs:213-244`); the migration must move to a by-reference build. Slice 6 work, listed in the DOM §1.6 checklist style.

---

## K2 — the `Grid::update` bound

### K2.1 Decision

**Two entry points, with the base trait owning the base name.**

```rust
impl<'a> Grid<'a> {
    /// Read-only navigation, selection, sorting, copy, fetch-more, filter and cell actions.
    /// `&M`: a read-only grid CANNOT mutate its model — a compile-time fact, not a runtime refusal.
    pub fn update<M: GridModel + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &M) -> Response<GridAction>;

    /// Everything `update` does, plus the inline edit lifecycle: begin, cycle, commit, cancel, blur.
    pub fn update_editable<M: GridEditor + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;

    /// One draw for both. Bound is the base trait, symmetric with `update`.
    pub fn draw<M: GridModel + ?Sized>(
        &self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
}
```

Two consequential corrections come with it:

```rust
pub trait GridModel {
    type Row;
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> ItemKey;
    fn cell(&self, row: usize, col: usize) -> CellRef<'_>;
    fn row_decor (&self, row: usize)             -> RowDecor<'_>  { RowDecor::default() }
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> { CellDecor::default() }
    fn total(&self) -> RowTotal { RowTotal::Unknown }
    fn has_more(&self) -> bool { false }
    // ── moved down from GridEditor: `draw` renders the reason and must see it ──
    fn read_only_reason(&self) -> Option<&str> { None }
    // ── absorbed from GridCellActions: `draw` paints the affordance and must see it ──
    fn actions(&self, _row: usize, _col: usize) -> &[CellAction] { &[] }
}

pub trait GridEditor: GridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent;
    fn apply_cycle(&mut self, row: usize, col: usize);
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
}
```

**`Grid::editable(bool)` (`COMPONENT_ARCHITECTURE.md:2081`) is deleted.** Capability is chosen by the entry point; the boolean was the soup §13 forbids (`:1131`) and it could contradict the model's own `is_editable`.

### K2.2 Why `GridCellActions` and `read_only_reason` must move

This is the finding that settles the shape, and it is independent of which option is chosen.

* **[F]** `Grid::draw` is bound `M: GridModel` (`COMPONENT_ARCHITECTURE.md:1084`, restated `:2084`). **[F]** `read_only_reason` is declared on `GridEditor` (`:1087`). **[F]** the read-only reason is rendered — today it is a `DataGrid` field consumed by drawing (`src/widgets/grid.rs:348`; DOM §1.6 capability 3 keeps it, `docs/audit/domain-boundary-audit.md:162`). A method on `GridEditor` is **unreachable from `draw<M: GridModel>`**. As written, §12.3 cannot render the reason at all.
* **[F]** Same for cell actions: `GridCellActions` is a third, separate trait (`:1089-1091`), yet the `→` affordance it describes is **painted** (`src/widgets/grid.rs:1858-1868`, DOM §1.6 capability 16) and its hot zone is registered in `draw`. `draw<M: GridModel>` cannot see it. Adding a second bound (`M: GridModel + GridCellActions`) to `draw` would force *every* model — including the six read-only Structure-tab models — to implement it.
* Absorbing both as defaulted methods on `GridModel` matches the precedent already in §12.3 for `row_decor`, `cell_decor`, `total`, `has_more` (`:1074-1077`), and matches §12.2's "decoration supplied by the owner, never derived inside the component" (`:1038`).

### K2.3 Evaluation against the required criteria

**§13 conventions.** "No boolean parameter soup; typed enums for semantically different modes" (`:1131`). Read-only and editable are semantically different modes; two typed entry points are the sanctioned remedy, and they mirror the vocabulary §13 already uses for exactly this distinction — `.disabled(bool)` / `.read_only(bool)` with *read-only stays in the ring, disabled does not* (`:1117`), and `Focusability::{Focusable, FocusableReadOnly}` (`:566`). "One predictable vocabulary" is served by `update` and `draw` carrying the **same** bound and the same `&M` shape, so a navigating screen writes one bound in both phases.

**TablePro read-only-with-reason grids.** **[F]** Views / no-PK tables set a reason on the *same* adapter type that edits elsewhere (`tabs.rs:394-403`); **[F]** result grids do the same when the source table is unknown, plus `local_sort = true` (`tabs.rs:2062-2068`; DOM §5.4, `docs/audit/domain-boundary-audit.md:456`). These call `update_editable` with an adapter whose `is_editable` returns `false` and whose `read_only_reason` returns `Some` — a *runtime* property of an editor-capable model, which is what it is today. No wrapper, no second type, no `Refuse` string per attempt.

**Showcase demo grids, fixtures, Structure tab.** **[F]** `crates/tui/tests/fixtures/grid_model.rs` is explicitly "a test-only model" (`:3115`); **[F]** DOM §2.12 turns the Structure tab into "six `GridModel`s over catalog data" (`docs/audit/domain-boundary-audit.md:318`); the showcase grid page is a display. These implement `GridModel` only — four required methods — and call `update`. Under option (a) each would have to implement a trait literally named `GridEditor`, which is a vocabulary lie in the type that carries it.

**`GridCellActions` composition.** Resolved by K2.2: it becomes `GridModel::actions` with a `&[]` default, so FK-follow affordances work on read-only grids (a view can carry FK columns) under *both* entry points and are visible to `draw`. Under any option that keeps it a separate trait, it is unreachable from `draw`.

**Conformance.** **[F]** `Fixture` already carries a `read_only: bool` knob (`:1423`). One `GridCase` registration selects the entry point from that knob, so both paths run the full 20-case matrix from a single registration, and case 7 (`draw_does_not_commit_or_cancel`, `:1457`) is exercised in both. Under a single `M: GridEditor` entry point, the read-only path is a defaulted refusal rather than a distinct code path, so case 7's read-only variant asserts nothing new.

### K2.4 Rejected alternatives

**(a) `Grid::update<M: GridEditor>` with defaulted refusals on `GridEditor`.**
* Every read-only model must implement a trait named "Editor"; §13's "one predictable vocabulary" is about the names the reader sees, and this makes the name lie at ~10 of the ~14 known model sites.
* `update` would take `&mut M` while `draw` takes `&M`, on the *same* model, for a grid that cannot edit — the `&mut` conveys a capability that does not exist and blocks a shared borrow in the same screen frame for no reason.
* A default `edit_intent` must synthesise `EditIntent::Refuse { reason }` from `read_only_reason()`, allocating a `String` on a refused keystroke; more importantly, an editable model that *forgets* to override `commit_cell` inherits a default that silently refuses — a wrong-but-compiling grid. The two-entry-point shape has no such default.
* It does not fix the K2.2 reachability problem; `read_only_reason` and `actions` would still be invisible to `draw`.

**(c) blanket `impl<M: GridModel> GridEditor for ReadOnly<M>`.**
* `update` needs `&mut ReadOnly<M>`. Deriving one from a `&mut M` requires a `repr(transparent)` reinterpretation — `unsafe`, forbidden by `#![forbid(unsafe_code)]` (`:3324`, `architecture::no_unsafe` `:1626`). The alternative is storing `ReadOnly<Model>` in app state, which changes the app's ownership shape and contradicts §21 item 1's "the model is a phase parameter, never a field".
* It needs a *second* blanket for the actions trait (`impl GridCellActions for ReadOnly<M> where M: GridCellActions`), and the two blankets plus a hand-written `impl GridEditor for TableModel` risk coherence conflicts.
* Net effect is still "`update` always requires `GridEditor`", i.e. option (a) plus a wrapper type — strictly more machinery for the same vocabulary problem.

**(d) naming it `update_readonly` / `update`** (the literal phrasing of the open question). Rejected in favour of `update` / `update_editable`, which is also the phrasing §21 item 1 itself used (`:3411`). Reasons: `update` and `draw` then share one bound; read-only call sites outnumber editable ones across the three consumers; and the capability is named at the point where the extra capability is used, matching `read_only`/`disabled` conventions.

### K2.5 Invariants

* **G1** `Grid::update` takes `&M`. A read-only grid is structurally incapable of mutating its model. `draw` and `update` carry the same bound and the same shared borrow.
* **G2** `Grid::update_editable` is the **only** place `GridEditor`'s `&mut self` methods are reachable. With `draw`'s `&self`/`&GridState`, "rendering stages a database mutation" (**[F]** `src/widgets/grid.rs:1518-1520`) is unrepresentable — the §21 item 1 / B15 rationale is preserved intact.
* **G3** `read_only_reason` and `actions` live on `GridModel` because `draw` renders them (K2.2).
* **G4** No boolean capability parameter exists on `Grid`; `Grid::editable(bool)` is deleted.
* **G5** `EditIntent::External` emits `GridAction::EditRequested(item, col)` and begins no inline edit (§21 item 30 A8, `:3652`) — reachable only from `update_editable`.
* **G6** An inline editor registers its `Control` region **after** the grid's cell `Part` region and wins the click (§21 item 30, `:3650`) — unchanged by this decision.
* **G7** Both entry points call `GridState::reconcile` before emitting any action (§12.2, `:1056`).

### K2.6 Acceptance conditions (executable)

```bash
cargo test -p junie-tui --lib grid::
cargo test -p junie-tui --test conformance conformance::grid::
cargo test -p junie-tui --test architecture
cargo test -p tablepro tablepro::grid_adapter_keeps_every_pending_change_capability
cargo test -p junie-tui --test perf --release -- --test-threads=1 grid_
! rg -n 'fn editable\(' crates/tui/src/components/grid.rs
! rg -n 'trait GridCellActions' crates/tui/src
```

Named tests:

* `grid::read_only_update_takes_a_shared_model` — a `trybuild` compile-fail proving `Grid::update` cannot reach `commit_cell`/`apply_cycle`.
* `grid::update_editable_commits_through_the_editor` — begin → type → Enter → `commit_cell` observed once; a failing `commit_cell` leaves the editor open with the returned `FieldError`.
* `grid::read_only_reason_is_rendered_from_a_grid_model` — a model implementing **only** `GridModel` renders its reason (fails today's §12.3 as written).
* `grid::cell_actions_affordance_is_painted_for_a_read_only_model` — the `→` glyph and its hot zone appear for a `GridModel`-only model; a click emits `GridAction::CellAction(item, col, key)`.
* `grid::edit_intent_inline_cycle_external_refuse` *(retained, `:1368`)* — now reached only via `update_editable`.
* `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row` *(retained)*.
* `grid::click_inside_an_active_inline_edit_goes_to_the_editor` *(retained, `:1368`)*.
* `conformance::grid::draw_does_not_commit_or_cancel` — driven for both entry points from `Fixture.read_only`.
* `conformance::grid::item_identity_survives_reorder` — both entry points.
* `architecture::no_boolean_capability_parameter_on_grid` — grep, above.
* `tablepro::view_grid_is_read_only_with_a_reason` and `tablepro::result_grid_sorts_locally_and_refuses_edits` — DOM §1.6 capabilities 3 and 4 (`docs/audit/domain-boundary-audit.md:162-163`).

### K2.7 Risks

1. **A model that later becomes editable changes call site, not type.** A screen upgrading a grid from read-only to editable must switch `update` → `update_editable` and hold `&mut`. This is the intended, visible cost; a defaulted-refusal design hides it and lets a half-implemented editor ship.
2. **Two entry points means two `GridAction` paths to keep in sync.** Mitigated structurally: `update_editable` is implemented as `update`'s body plus the edit arms, sharing one private `fn navigate(...)`, and conformance runs the whole matrix through both.
3. **Moving `read_only_reason` and `actions` onto `GridModel` widens the base trait to 9 methods (5 defaulted).** Acceptable — it is the same shape §12.3 already chose for `row_decor`/`cell_decor`/`total`/`has_more` — and it is the only way `draw` can render either.
4. **`GridCellActions` deletion is a documented API change** relative to §12.3 (`:1089-1091`) and DOM §1.5 (`docs/audit/domain-boundary-audit.md:147-149`). It must be recorded as an amendment in §12.3 and mirrored in `REFACTORING_STATE.md` before work package 4I starts, per the change-control rule at `COMPONENT_ARCHITECTURE.md:3`.

---

## Sequencing

Neither decision blocks Slice 3 (§21 item 1 already records this for K2, `:3411`). K1 blocks **4F**, which is wave 2 and depends on 4B; K2 blocks **4I**, also wave 2 (`:3112-3119`). Both should be recorded in `COMPONENT_ARCHITECTURE.md` — K1 as new §15.1 plus a §17.0 A10 block and the §16.1 test names; K2 as an amendment to §12.3, §17.0 A7 and §16.1 — and mirrored in `REFACTORING_STATE.md` before wave 2 begins. Both are covered by `xtask doc-check` (§21 item 34) once written, so every identifier above must resolve against the built rustdoc-json or sit on its printed allow-list.
