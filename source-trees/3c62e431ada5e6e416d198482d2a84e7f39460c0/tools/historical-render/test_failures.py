"""Exercise fail-closed archive generation without modifying repository pins."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--font-archive', type=Path, required=True)
    args = parser.parse_args()
    source = Path(__file__).resolve().parent
    repo = source.parents[1]
    original = json.loads((source / 'pins.json').read_text())
    cases = [
        ('wrong-font', lambda p: p['font_archive'].update(sha256='0' * 64), 'hash mismatch: font archive'),
        ('wrong-renderer', lambda p: p['renderer_hashes'].update({'ansi2html.py': '0' * 64}), 'hash mismatch: ansi2html.py'),
        ('missing-source', lambda p: p.update(renderer_source='0' * 40), 'fatal:'),
        ('missing-recipe', lambda p: p['captures'].pop(), 'missing, duplicate or unexpected historical recipe'),
        ('duplicate-recipe', lambda p: p['captures'].append(p['captures'][0]), 'missing, duplicate or unexpected historical recipe'),
        ('wrong-input', lambda p: p['captures'][0]['inputs']['ansi'].update(sha256='0' * 64), 'hash mismatch:'),
        ('wrong-output', lambda p: p['captures'][0]['outputs']['png'].update(sha256='0' * 64), 'hash mismatch:'),
        ('stale-output', lambda p: None, 'output must not exist'),
    ]
    for name, mutate, diagnostic in cases:
        with tempfile.TemporaryDirectory(prefix='historical-failure-') as temporary:
            root = Path(temporary)
            shutil.copyfile(source / 'regenerate.py', root / 'regenerate.py')
            pins = json.loads(json.dumps(original))
            mutate(pins)
            (root / 'pins.json').write_text(json.dumps(pins))
            output = root / 'output'
            if name == 'stale-output':
                output.mkdir()
                (output / 'sentinel').write_text('preserved')
            result = subprocess.run([
                sys.executable, str(root / 'regenerate.py'), '--repo', str(repo),
                '--font-archive', str(args.font_archive.resolve()), '--output', str(output),
            ], capture_output=True, text=True, timeout=60)
            if result.returncode == 0 or diagnostic not in result.stderr:
                raise AssertionError(f'{name}: expected rejection missing\n{result.stderr}')
            if name == 'stale-output':
                assert (output / 'sentinel').read_text() == 'preserved'
                assert len(list(output.iterdir())) == 1
            else:
                assert not output.exists(), f'{name}: partial output published'
            print(f'PASS {name}', flush=True)
    print('8/8 fail-closed mutations detected')


if __name__ == '__main__':
    main()
