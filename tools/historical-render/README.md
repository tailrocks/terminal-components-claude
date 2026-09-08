# Historical image regeneration

These are regenerated renderings of the 499 preserved historical ANSI captures,
not recovered original PNG/HTML files or pinned-Holla acceptance snapshots.
The application/renderer source is `d5e7075f436f0e437c7d12cf3d1e638e763b26f6`;
the immutable capture/recipe archive is main base
`c12cad8728755cd2d03eefdd8e02891143fca86d`. The manifest header revision has the
same application/build inputs; see the main-Holla baseline audit.

From a clean checkout with full Git history and Python:

```sh
python3 -m venv /tmp/historical-render-venv
/tmp/historical-render-venv/bin/pip install -r tools/historical-render/requirements.txt
/tmp/historical-render-venv/bin/python tools/historical-render/regenerate.py --output /tmp/historical-renderings
```

The output must not exist. Failed generation never publishes partial results.
The script downloads the pinned Nerd Fonts 3.5.1 JetBrains Mono archive and checks
the upstream release SHA-256 plus all four individual font hashes. For offline
use, pass `--font-archive PATH` with that exact archive. Source blobs, all 1,497
input artifacts, recipe identities, viewports and all 998 generated files must
match `pins.json`. Original tracked captures remain untouched.

Font source: [Nerd Fonts 3.5.1 release](https://github.com/ryanoasis/nerd-fonts/releases/tag/v3.5.1),
[release checksums](https://github.com/ryanoasis/nerd-fonts/releases/download/v3.5.1/SHA-256.txt).
The archive's OFL.txt accompanies generated artifacts. The four font files match
the installed fonts used for initial regeneration byte for byte.

The historical renderer's limited Unicode/modifier behavior is intentionally
preserved. HTML browser font resolution is not pinned by its original CSS.
PNG output verification fails on any platform/rasterizer drift; do not replace
hashes to make another environment pass. Initial regeneration used Python 3.14
and the macOS arm64 Pillow 11.3.0 wheel. No original executable hash is available.
Independent review and installation into historical baseline paths remain pending.

Fail-closed qualification (eight mutations: font, renderer, source, missing/duplicate
recipe, input, output and stale output):

```sh
/tmp/historical-render-venv/bin/python tools/historical-render/test_failures.py --font-archive /path/to/JetBrainsMono.tar.xz
```

Root validation: all 499 recipes reproduced all 998 expected output hashes;
eight mutations failed without publishing partial output or changing existing
sentinel files. The initial external manifest SHA-256 is
`24e492a6aee8b1719b16e10d72ee72ccf2c14a81bb702917e941d5f5a3959608`.
An independent clean detached checkout at `c051c8f` also downloaded the font
archive and reproduced all 998 hashes with no source changes. This root-operated
clean-checkout check is separate from the pending independent reviewer approval.
