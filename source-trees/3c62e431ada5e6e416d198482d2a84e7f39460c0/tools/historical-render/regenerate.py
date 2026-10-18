#!/usr/bin/env python3
"""Rebuild missing historical images from immutable ANSI, never candidate output."""
import argparse
import csv
import hashlib
import importlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

import PIL


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verified(data, expected, label):
    if sha(data) != expected:
        raise ValueError(f'hash mismatch: {label}')
    return data


def blob(repo, revision, name):
    return subprocess.check_output(['git', 'show', f'{revision}:{name}'], cwd=repo)


def regenerate(repo, output, archive_path):
    pins = json.loads(Path(__file__).with_name('pins.json').read_text())
    if PIL.__version__ != pins['pillow']:
        raise ValueError(f"requires Pillow {pins['pillow']}; found {PIL.__version__}")
    if output.exists():
        raise ValueError('output must not exist; refusing stale artifact reuse')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.historical-render-', dir=output.parent) as scratch:
        root = Path(scratch)
        archive = pins['font_archive']
        if archive_path:
            font_data = archive_path.read_bytes()
        else:
            with urllib.request.urlopen(archive['url'], timeout=120) as response:
                font_data = response.read()
        verified(font_data, archive['sha256'], 'font archive')
        font_dir = root / 'fonts'
        font_dir.mkdir()
        with tarfile.open(fileobj=io.BytesIO(font_data), mode='r:xz') as fonts:
            for font in pins['fonts'].values():
                matches = [m for m in fonts.getmembers()
                           if m.isfile() and Path(m.name).name == font['name']]
                if len(matches) != 1:
                    raise ValueError(f"missing or duplicate font: {font['name']}")
                data = fonts.extractfile(matches[0]).read()
                verified(data, font['sha256'], font['name'])
                (font_dir / font['name']).write_bytes(data)
            license_data = fonts.extractfile(archive['license_member']).read()
        source = root / 'renderer'
        source.mkdir()
        for name, expected in pins['renderer_hashes'].items():
            data = blob(repo, pins['renderer_source'], f'tools/{name}')
            (source / name).write_bytes(verified(data, expected, name))
        sys.path.insert(0, str(source))
        html = importlib.import_module('ansi2html')
        png = importlib.import_module('ansi2png')
        # Environment adapter only: identical font bytes, unchanged renderer logic.
        png.FONTS = {kind: str(font_dir / value['name'])
                     for kind, value in pins['fonts'].items()}
        recipes_data = blob(repo, pins['capture_archive_revision'], 'parity/recipes.tsv')
        verified(recipes_data, pins['recipes_sha256'], 'historical recipes')
        recipes = list(csv.DictReader(io.StringIO(recipes_data.decode()), delimiter='\t'))
        expected = {row['recipe_id']: row for row in pins['captures']}
        ids = [row['recipe_id'] for row in recipes]
        if (len(ids) != 499 or len(set(ids)) != 499
                or len(pins['captures']) != 499 or set(ids) != set(expected)):
            raise ValueError('missing, duplicate or unexpected historical recipe')
        generated = root / 'generated'
        generated.mkdir()
        for index, recipe in enumerate(recipes, 1):
            name = recipe['recipe_id']
            contract = expected[name]
            cols, rows = map(int, recipe['viewport'].split('x'))
            if [cols, rows] != contract['viewport']:
                raise ValueError(f'viewport mismatch: {name}')
            inputs = {}
            for ext in ('ansi', 'txt', 'cursor'):
                entry = contract['inputs'][ext]
                if recipe[f'expected_{ext}'] != entry['path']:
                    raise ValueError(f'input path mismatch: {name}')
                data = blob(repo, pins['capture_archive_revision'], entry['path'])
                inputs[ext] = verified(data, entry['sha256'], entry['path'])
            text = inputs['ansi'].decode('utf-8', errors='replace')
            fields = inputs['cursor'].decode().split()
            if len(fields) != 3 or fields[2] not in ('0', '1'):
                raise ValueError(f'invalid historical cursor: {name}')
            cursor = tuple(map(int, fields[:2])) if fields[2] == '1' else None
            (generated / f'{name}.html').write_text(html.convert(text, cols, rows), encoding='utf-8')
            png.render(text, cols, rows, str(generated / f'{name}.png'), cursor=cursor)
            for ext in ('html', 'png'):
                artifact = generated / f'{name}.{ext}'
                verified(artifact.read_bytes(), contract['outputs'][ext]['sha256'], artifact.name)
            if index % 25 == 0:
                print(f'{index}/499 verified', flush=True)
        (generated / 'FONT-OFL.txt').write_bytes(license_data)
        (generated / 'PROVENANCE.json').write_text(json.dumps({
            'classification': pins['classification'],
            'pins_sha256': sha(Path(__file__).with_name('pins.json').read_bytes()),
            'renderer_source': pins['renderer_source'],
            'capture_archive_revision': pins['capture_archive_revision'],
            'python': sys.version, 'pillow': PIL.__version__,
            'verified_recipes': 499, 'verified_renderings': 998,
            'limitations': pins['limitations'],
        }, indent=2, sort_keys=True) + '\n')
        os.rename(generated, output)
    print(f'499 recipes / 998 historical renderings verified: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--font-archive', type=Path, help='optional offline, hash-verified archive')
    args = parser.parse_args()
    regenerate(args.repo.resolve(), args.output.resolve(), args.font_archive)
