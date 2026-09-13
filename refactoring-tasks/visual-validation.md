# Visual validation during refactoring

Live visual oracle is the committed grouped store. Use it to prove a refactor
did not change user-visible output. Do not use the deleted `shots/` corpus.

```text
snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.{ansi,txt,png,html}
```

Each terminal size is its own folder. Color is the leaf. Taxonomy:
`docs/baseline/snapshots-v2.md`. Suite: `tests/visual_baseline/`.

## Gate

Fail-closed. Tests never bless. Candidates never write `snapshots/` and never
run `tuisnap accept`.

```sh
# full suite (skip HTML report rebuild)
cargo test --test visual_baseline -- --ignored --skip rebuild_review_html --test-threads=4

# one family
cargo test --test visual_baseline holla_ -- --ignored --skip rebuild_review_html
cargo test --test visual_baseline store_integrity -- --exact
```

`matched` = no UI/UX drift. `cells-differ` / `pixels-differ` = regression.
Inspect `target/tuisnap/diff/<name>.png` and the first-difference notes.

`verify.toml` `forbidden_paths` includes `snapshots` and `shots` on every task.
`CHK-004` compare still uses the sealed oracle bundle when that product exists;
this store is the additional live regression gate for product edits on holla.

## Bless (host only)

Only the host blesses an *intended* visual change:

```sh
ln -sfn target/tuisnap/actual snapshots.actual
cargo run --manifest-path ~/Projects/tui-snap/Cargo.toml --release -- \
  accept --grouped --store snapshots --name <group/…/size/color>
```
