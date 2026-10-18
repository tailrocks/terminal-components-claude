# Historical render generator

This tool regenerates archived HTML and PNG renderings from pinned historical ANSI captures. Outputs are historical reference artifacts; they are not current application screenshots or proof of product parity.

Run from the repository root with Python and the pinned Pillow dependency:

```sh
python3 -m venv /tmp/historical-render-venv
/tmp/historical-render-venv/bin/pip install -r tools/historical-render/requirements.txt
/tmp/historical-render-venv/bin/python tools/historical-render/regenerate.py \
  --output /tmp/termrock-historical-renderings
```

The destination must not exist. The generator validates source blobs, capture inputs, recipes, font archive and individual font hashes against `pins.json`; it refuses stale or partial output. Pass `--font-archive PATH` to use a local archive with the pinned hash. The output includes the upstream OFL license; the retained license and provenance are in [`docs/historical-regeneration`](../../docs/historical-regeneration/).

The fail-closed harness takes the pinned font archive explicitly:

```sh
/tmp/historical-render-venv/bin/python \
  tools/historical-render/test_failures.py --font-archive /path/to/JetBrainsMono.tar.xz
```

This workflow writes only to the requested new output directory. It does not bless, install, or update repository baselines.
