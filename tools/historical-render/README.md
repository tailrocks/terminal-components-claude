# Historical render recovery

This tool produces **`regenerated-historical-renderings-not-originals`**: HTML
and PNG renderings reconstructed from pinned historical ANSI captures with
their cursor inputs. The associated text sidecars are hash-checked as part of
the recipe but are not used to render the images. The tool does not launch the
current application, compare a candidate scene, recover the original
executable or image bytes, establish current product parity, or satisfy
pinned-Holla acceptance. It does not authorize changing a baseline or other
oracle.

The rendering recipe is pinned in [`pins.json`](pins.json). It names two
separate immutable Git sources: the historical application/renderer commit
`d5e7075f436f0e437c7d12cf3d1e638e763b26f6` and the capture archive commit
`c12cad8728755cd2d03eefdd8e02891143fca86d`. The latter contains the recipes
and captured inputs. Keep both commits available in the Git object store used
for regeneration; the script reads them with `git show` and does not fetch
missing objects. This set contains 499 unique recipes and 998 HTML/PNG
renderings. It is distinct from the 704-key current-versus-historical Showcase
scene comparison.

The pinned limitations remain:

- The original executable hash and original image bytes are unavailable.
- Historical Unicode and modifier-rendering limitations are retained.
- HTML font resolution depends on the browser environment.
- These artifacts are not pinned-Holla acceptance evidence.

The retained [historical provenance](../../docs/historical-regeneration/PROVENANCE.json)
and [font license](../../docs/historical-regeneration/FONT-OFL.txt) are
qualified historical records, not evidence of a current application capture.

## Regenerate to a fresh external destination

The command requires `--output`. `--repo` optionally selects a Git object
store and defaults to the repository root inferred from the script location.
`--font-archive` optionally supplies a local archive. The tracked
[`requirements.txt`](requirements.txt) pins Pillow to 11.3.0, and the script
rejects a different Pillow version. These setup commands are examples only;
they were not run as part of this documentation change, and they do not
qualify the current host's Python installation:

```sh
python3 -m venv /tmp/historical-render-venv
/tmp/historical-render-venv/bin/python -m pip install \
  -r tools/historical-render/requirements.txt
```

For this recovery workflow, choose a **new destination outside the checkout**.
The script refuses an output path that already exists, but it does not enforce
an external-path allowlist; do not point it at a repository baseline or other
tracked data. For example, when a local copy of the pinned font archive is
available:

```sh
/tmp/historical-render-venv/bin/python tools/historical-render/regenerate.py \
  --repo . \
  --font-archive /path/to/JetBrainsMono.tar.xz \
  --output /tmp/termrock-historical-renderings
```

If `--font-archive` is omitted, the script downloads the archive from the URL
recorded in `pins.json`. Either way, it checks the archive hash and the hashes
of the selected font files. Supplying a local archive avoids that generator
network request. Installing Pillow is a separate operation: `pip` may use the
network and may write its cache as well as the virtual environment.

The destination is not the only filesystem path affected. The script creates
missing destination parents, creates a temporary sibling directory beneath
the destination parent, builds and verifies the complete output there, and
renames the generated directory to the destination. A failure before that
rename does not publish a partial destination; temporary data is removed
when the temporary-directory context exits. The script does not automatically
install or bless repository baselines, but the caller can choose a repository
path, which is why this workflow requires an external destination.

## What is verified

The tracked `pins.json` supplies the expected identities and hashes. The
script loads the renderer files from the pinned renderer Git commit and checks
their hashes before importing them from its temporary directory. It reads the
recipe table from the pinned capture commit, verifies its hash, and checks the
499 unique recipe IDs, recipe-to-pin membership, viewport, input paths and
input hashes, cursor record shape, and each generated HTML/PNG digest. It also
hash-checks the archive and each selected font. The script does not
independently authenticate the `pins.json` file or sandbox Python, Git,
filesystem, or environment access; review the tracked pins and their Git
provenance as part of using these records.

The output directory contains the 998 verified renderings, `FONT-OFL.txt`,
and `PROVENANCE.json`. The provenance file records the pins hash, source
commits, Python and Pillow versions, counts, and the limitations above.
Verification is against this historical rendering recipe; it is not a claim
about the original capture executable or the current renderer.

## Fail-closed regression harness

The harness uses a supplied font archive and temporary copies of the script
and pins. Its eight mutations cover a wrong archive hash, wrong renderer hash,
missing source commit, missing recipe, duplicate recipe, wrong input hash,
wrong output hash, and an already-existing destination. It expects each case
to fail without publishing partial output; the stale-destination case also
checks that its sentinel remains unchanged. This command is documented for
operators but was not run for this documentation change:

```sh
/tmp/historical-render-venv/bin/python \
  tools/historical-render/test_failures.py \
  --font-archive /path/to/JetBrainsMono.tar.xz
```
