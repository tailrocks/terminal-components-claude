// Read-only reproduction of the sections 0–29 amendment inventory.
const fs = require('node:fs');
const cp = require('node:child_process');
const crypto = require('node:crypto');
const architectureRows = fs.readFileSync('docs/refactoring-plan/history-revision-index.tsv', 'utf8').trim().split('\n').slice(1).map(line => line.split('\t')).filter(row => row[2] === 'COMPONENT_ARCHITECTURE.md');
if (architectureRows.length !== 96 || !architectureRows[10][0].startsWith('3ed377e3')) throw new Error('Architecture index changed; reassess the explicit ownership boundary.');
const rows = architectureRows.slice(11);
const hash = text => crypto.createHash('sha256').update(text).digest('hex');
const read = revision => cp.execFileSync('git', ['show', `${revision}:COMPONENT_ARCHITECTURE.md`], { encoding: 'utf8', maxBuffer: 8e6 });
const prefix = text => text.split(/(?=^## §?30[ .])/m)[0];
const seen = new Map();
const seenHunks = new Map();
if (process.argv[2] === 'hunk-inventory') console.log('commit\tparent\thunk\tpayload_sha256\tfirst_identical_payload');
for (const row of rows) {
  const [commit, parent, path, beforeBlob, afterBlob] = row;
  const before = prefix(read(parent));
  const after = prefix(read(commit));
  const key = `${hash(before)}:${hash(after)}`;
  const representative = seen.get(key);
  seen.set(key, representative || `${commit.slice(0, 8)}/${parent.slice(0, 8)}`);
  if (process.argv[2] === 'inventory') {
    console.log([commit,parent,beforeBlob,afterBlob,hash(before),hash(after),before===after?'unchanged':representative?`identical_prefix_pair:${representative}`:'unique_changed_prefix_pair'].join('\t'));
  } else if (!representative && before !== after) {
    const full = cp.execFileSync('git', ['diff', '--no-ext-diff', ...(process.env.AMENDMENT_WORD ? ['--word-diff=porcelain'] : []), (process.env.AMENDMENT_NOVEL || process.argv[2] === 'hunk-inventory') ? '--unified=0' : '--unified=3', parent, commit, '--', path], { encoding: 'utf8', maxBuffer: 8e6 });
    let chunks = full.split(/(?=^@@ )/m).slice(1).filter(chunk => {
      const match = chunk.match(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/);
      return +match[1] <= before.split('\n').length - 1 || +match[3] <= after.split('\n').length - 1;
    });
    if (process.env.AMENDMENT_NOVEL || process.argv[2] === 'hunk-inventory') chunks = chunks.filter(chunk => {
      const payload = chunk.split('\n').filter(line => /^[+-]/.test(line)).join('\n');
      const existed = seenHunks.has(payload);
      const location = `${commit}/${parent}/${chunk.split('\n')[0].match(/^@@.*?@@/)[0]}`;
      if (!existed) seenHunks.set(payload, location);
      if (process.argv[2] === 'hunk-inventory') console.log([commit,parent,chunk.split('\n')[0].match(/^@@.*?@@/)[0],hash(payload),seenHunks.get(payload)].join('\t'));
      return !existed;
    });
    if (process.argv[2] === 'hunk-inventory') continue;
    if (process.argv[2] && !process.argv.slice(2).some(value => commit.startsWith(value))) continue;
    const lines = chunks.join('').split('\n').filter(line => !process.env.AMENDMENT_WORD || /^[-+@]/.test(line));
    const start = Number(process.env.AMENDMENT_START || 0);
    const end = Number(process.env.AMENDMENT_END || lines.length);
    console.log(`\nEDGE ${commit} PARENT ${parent} DELTA LINES ${start + 1}–${Math.min(end, lines.length)} OF ${lines.length}\n${lines.slice(start, end).join('\n')}`);
  }
}
