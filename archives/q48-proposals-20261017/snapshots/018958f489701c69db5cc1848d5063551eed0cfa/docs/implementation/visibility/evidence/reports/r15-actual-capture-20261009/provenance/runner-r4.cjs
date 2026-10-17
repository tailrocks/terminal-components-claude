'use strict';
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { spawn, spawnSync, execFileSync } = require('child_process');
const evidenceRoot = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r15-r2-d990-20261009-01';
const runRoot = path.join(evidenceRoot, 'attempt-02');
const repo = '/Users/donbeave/Projects/tailrocks/terminal-components-claude';
const packageRoot = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/package';
const freezePath = path.join(evidenceRoot, 'candidate-freeze.json');
const cargo = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo';
const rustc = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustc';
const rustdoc = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustdoc';
const node = '/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node';
const nextest = '/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest';
const cargoHome = '/private/tmp/termrock-rust-tool-test-execution/e2e-r2-private-cache-20261009/cargo-home';
const cacheVerificationPath = '/private/tmp/termrock-rust-tool-test-execution/e2e-r2-private-cache-20261009/cache-verification.json';
const cacheVerificationSha256 = 'f6418704b2917bfbc2646a2df73c34e69a6cf3a3e30b9e89ab83bcf41edfbf01';
const prefetchCompletionPath = '/private/tmp/termrock-rust-tool-test-execution/e2e-r2-private-cache-20261009/prefetch-completion.json';
const prefetchCompletionSha256 = '02874b9da5be4e3ca657382361adfe2e7bab69417001613431c3a50b7d88c82a';
const expectedLockSha256 = '6c24ba612e204973dbf9414af15c8d5ee33f2d2ae6bc11bbcaff97a9c9f1e73e';
const targetDir = '/private/tmp/termrock-e2e-target-vis02-r2-privatecache-20261009';
const captureTestBinaryName = 'holla_tag_capture-fa2a1ab31e8cb087';
const captureTestBinaryPath = path.join(targetDir, 'debug/deps', captureTestBinaryName);
const captureTestBinarySha256 = '7014b46f1facde2ac83894702619dec1565453a5163a87eb5289aa56ce3c4060';
const rustup = '/Users/donbeave/.cargo/bin/rustup';
const python3 = '/usr/bin/python3';
const git = '/usr/bin/git';
const toolHashes = { cargo: '6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d', rustc: '766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941', rustdoc: '8ce1ae78157758d4d75eee1355ed333f27e903ab6b0904e09d326009b333c2ee', nextest: '7a558b157d164ab4fb6cb1a48cbac5a57b7b8ad99f5d3492eb6eda64faf91df0', node: '9d050fd455b56426e25d4d603c7c501cbb2630348e836cf221dcce748e90588a', rustup: 'ec1b9233e7f72990ecd8e62063fa7f6c3dfc2bec8e97f88bff165f9100ac696a', python3: '34129c71a01a74f7f3b2443521519b2e5447553fa187f5fcafaaf8c42cc192e2', git: '34129c71a01a74f7f3b2443521519b2e5447553fa187f5fcafaaf8c42cc192e2', clang: '1590ac950a3d627817d09ade5cb60b2115f17a72182a3141e010b4bcc482a0c9', linker: '374d9f0713bce2c8a9e1e82bde51615d5433516a10d99b63107eb6ee70542e4c', xcrun: 'bbe7671518a3905f9d3fce54312fbc77c2876b0d62aca74a0688fa869d31a8a9' };
const developerDir = '/Library/Developer/CommandLineTools';
const sdkRoot = '/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk';
const sdkResolvedRoot = '/Library/Developer/CommandLineTools/SDKs/MacOSX27.0.sdk';
const sdkSettingsJson = path.join(sdkResolvedRoot, 'SDKSettings.json');
const sdkSettingsPlist = path.join(sdkResolvedRoot, 'SDKSettings.plist');
const sdkHashes = { sdk_settings_json: '7b93ad7e534cc4b31c6a4e39d19b5e0288acf2168c2649479a796d1cb51939fb', sdk_settings_plist: '90a738dbf3fd88e7f6d35646e19d3a8a716923b9773ce1b67260466d9ceed370' };
const sdkToolPaths = { clang: '/Library/Developer/CommandLineTools/usr/bin/clang', linker: '/Library/Developer/CommandLineTools/usr/bin/ld', xcrun: '/usr/bin/xcrun' };
const rendererRoot = path.join(cargoHome, 'git/checkouts/tuiscotti-9317ad0850957efe/a47c9aa/crates/tuiscotti-render');
const rendererCommit = 'a47c9aaefb34e4c00026f99d8a8dd7ee5916b274';
const rendererFiles = {
  'Cargo.toml': '338eda5223a37c6dde03e5e7a900ec55a48fbfb09b5a9f14637816b272812c5b',
  'src/profile/fonts.rs': '6b27214acd3698fd2470c96bf7bf3b5b1290e79d4f925151551ed429c4331609',
  'src/profile/strict.rs': '4cb98ccc1dc996add327876c2e40ca470097390988ecb6d6753f54936f5943a0',
  'src/profile/strict_types.rs': '050fc00904dc868a2f106b795a83351e5bda47a806063c56a4281cccd721de07',
  'fonts/JetBrainsMonoNerdFontMono-Regular.ttf': 'f2a5ea6cfab397445ffab00c0370927b66d61e560a05db5db271b42006381c1a',
  'fonts/JetBrainsMonoNerdFontMono-Bold.ttf': 'bfcf9a917276ffc058867d87cbc8a5b2f1ab0f4b710e9170dc02763ccb80bd4b',
  'fonts/JetBrainsMonoNerdFontMono-Italic.ttf': '31efd6ead98746f5b0afa1ee6dba60267ad48db36428360bee327bec10621f97',
  'fonts/JetBrainsMonoNerdFontMono-BoldItalic.ttf': '9dba502e00e35209f6ed2a151c7376c051657b067cdebbc6e52d06cb9002cf31',
  'fonts/NotoSansCJKjp-subset.otf': '777bee41f0c6076c00ad919384359a6e396b8822cf9056041fca8fcf2759d897',
  'fonts/NotoSansSymbols-subset.ttf': '6f9cc93e71f8676361c5db286368be046e75d42c0b841afbf3f50da6bb0a2b8a',
  'fonts/NotoSansSymbols2-subset.ttf': 'e1d177a40af910100eceb0e825331e55f0cfd005bc0f26087fd4e58fbe60e6c5'
};
const expectedFreezeSha = '09429c5647d554bf53dfe66e814cea5d7ea8d47e7143d12dbbf3e689cdd13f52';
const expectedDigest = '037460649d3663f27d4c3af77c36d3519f3262fae82b99535396aa0eb65f0590';
const maxRawOutputBytes = 8 * 1024 * 1024;
const timeoutMs = 1200000;
const expectedTestCount = 1;
const runnerTemplatePath = '/private/tmp/termrock-rust-tool-test-execution/e2e-r2-contract14-privatecache-20261009/runner-v5.cjs';
const runnerTemplateSha256 = 'd7ca8a35303d0007c452b4917d922fc55942a0f6335d22045dcdc495b8045ae6';
const r13ReferenceRunnerPath = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r13-r5-f072-20261009-01/runner.cjs';
const r13ReferenceRunnerSha256 = '183218ad139982fba9a75481ae7ca10b1fd4e3d4dcd95cf32248c31d05f6d34a';
const r7PatternReferencePath = '/private/tmp/termrock-vis06-tag-holla-runner-20261009-r7-reuse-cache.cjs';
const r7PatternReferenceSha256 = '15011afd6843d61413779e6ebee364c0082d69b501cc5bfaac20252071d3087f';
const prefetchPackageRoot = '/private/tmp/termrock-vis02-suite-checkpoint-r2-20261009';
const tagRunId = 'tag-20261008T212348Z-25ce63f5a3b7';
const tagEvidenceRoot = '/private/tmp/termrock-vis06-tag-holla-tag-20261008T212348Z-25ce63f5a3b7/evidence/' + tagRunId;
const tagReceiptPath = path.join(tagEvidenceRoot, 'tag-builder-receipt.json');
const tagReceiptSha256 = '772208ace8cff5c1f1dc70a62ceb8771fcde5b429104636d679b2121a9cdc6ec';
const tagNestedReceiptSha256 = '89fd9cc1d03f5278c0bca0480e0dace110c154f0d36aed7bcc9113e2a260ba02';
const tagRunPath = path.join(tagEvidenceRoot, 'run.json');
const tagRunSha256 = '744f22edbe4f35dca773715bc6cf738f5fb0b24c83d5c8dd0edc3e9575a02bb1';
const tagClosurePath = '/private/tmp/termrock-vis06-tag-closure-tag-20261008T212348Z-25ce63f5a3b7/actual-evidence-seal.json';
const tagClosureSha256 = 'f5b4796e32625bcc96476a8ccd4b7b80740c9337faf4eaa69ccc1dbea5dfd86a';
const tagObject = '1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5';
const tagCommit = '4a79c0a2d40fca46fc406b77157ce3b3f12ec16b';
const tagTree = '0b1f13431fdfd6060cf9f45a114afa5a99cc6c26';
const tagSourceRoot = path.join(tagEvidenceRoot, 'source/oracle');
const tagBuildRoot = path.join(tagEvidenceRoot, 'targets/oracle');
const tagBinary = path.join(tagBuildRoot, 'aarch64-apple-darwin/release/holla');
const tagBinarySha256 = 'a3b93ef2c1b018cc2b337a7ed2657450669a9c9497aa35733bce2d09fe2e9bd5';
const validatorHelper = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/inputs/validator/subjects.py';
const validatorHelperSha256 = 'd990b77fbec0d01f242dbd191698939cbf5790d8dcd203337166c26688d85853';
const validatorSourceCommit = '23b0eeadc1fa2ba83f13e243b7e54af95e9c59de';
const validatorSourcePublicationPath = path.join(evidenceRoot, 'inputs/validator/source-publication.json');
const validatorSourcePublicationSha256 = '10b4744bbcea9a85e623b0db2701598d4563a8b2a243d68593deeff8a0721c86';
const validatorToolchainPath = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/inputs/validator/toolchain.json';
const validatorToolchainSha256 = 'd3bce7d5775bf755e3d0a6fb675a4eea3e62a2ffaf775a4874d91f7a43bff072';
const sourceReviewPath = path.join(evidenceRoot, 'inputs/reviews/r2-checkpoint-source-review.json');
const sourceReviewSha256 = '6189179c67b494ba158570250c8b006c87e269eb51c1f4616202aa13b8cade9e';
const candidateSourceReviewPath = path.join(evidenceRoot, 'inputs/reviews/d990-helper-pin-source-review.json');
const candidateSourceReviewSha256 = '393acd58eecba22b3448005b22913e9594470dd8c762816063c42ad83b097712';
const contractReviewPath = path.join(evidenceRoot, 'inputs/reviews/r2-contract14-actual-review.md');
const contractReviewSha256 = 'ca7236861d3efcfe616ebc4d5d7ade3566245a36402877049690a50b237de2a5';
const contractReceiptPath = path.join(evidenceRoot, 'inputs/reviews/r2-contract14-final-receipt.json');
const contractReceiptSha256 = 'e9b086621d2fd0cdb9f3edfcfc61eca6f14465628aaecefd116e479d11b482a5';
const runnerReviewPath = path.join(evidenceRoot, 'inputs/reviews/r2-runner-v5-review.md');
const runnerReviewSha256 = '6f139a7434a11f3d203ccf9f5cd419db3c4b3b2be85c9f84b04bcc25794b5329';
const actualRoot = '/private/tmp/termrock-tag-capture-r15-r4-output-20261009';
const actualOutputRoot = path.join(actualRoot, 'oracle-actual');
const captureReceiptPath = path.join(actualRoot, 'oracle-capture-receipt.json');
const launchGrantPath = path.join(evidenceRoot, 'launch-grant-r4.json');
const invocationManifestPath = path.join(evidenceRoot, 'invocation-manifest-r4.json');
const expectedSuite = {revision:'termrock-e2e-2026-10-09.1', caseSetDigest:'bd786b32e08bb1c88254caaa2e1203077363683ca3dcac0838b07dc7250cc8b5', profileDigest:'fc36250c315565fe5ef1b7e099b5a868398e76aaa6fd209fcf87c22824289af6', rendererHash:'e25cbd7fc819a7c73d69493755154f03bf8bc1afea3f87c9a426557e637c4db4'};
const startedWall = Date.now();
if (process.execPath !== node || process.version !== 'v24.20.0') throw new Error('runner must use the pinned Node binary and version');
function sha(data) { return crypto.createHash('sha256').update(data).digest('hex'); }
function shaFile(p) { return sha(fs.readFileSync(p)); }
function u64(n) { const b = Buffer.alloc(8); b.writeBigUInt64BE(BigInt(n)); return b; }
function inventory(freeze) {
  const root = packageRoot;
  const rootStat = fs.lstatSync(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) throw new Error('package root is not a real directory');
  if (fs.realpathSync(root) !== root) throw new Error('package root is not canonical');
  const found = [];
  function walk(dir, prefix) {
    for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
      const rel = prefix ? prefix + '/' + ent.name : ent.name;
      const parts = rel.split('/');
      if (parts.includes('target') || parts.includes('.git')) continue;
      const full = path.join(dir, ent.name);
      const st = fs.lstatSync(full);
      if (st.isSymbolicLink()) throw new Error('symlink: ' + rel);
      if (st.isDirectory()) walk(full, rel);
      else if (st.isFile()) found.push(rel);
      else throw new Error('non-regular path: ' + rel);
    }
  }
  walk(root, '');
  found.sort();
  const expectedPaths = freeze.files.map(function(x) { return x.path; }).sort();
  if (JSON.stringify(found) !== JSON.stringify(expectedPaths)) throw new Error('package file inventory changed');
  const digest = crypto.createHash('sha256');
  const rows = [];
  for (const rel of found) {
    const rec = freeze.files.find(function(x) { return x.path === rel; });
    const pinned = readPinnedBytes(path.join(root, rel), rec.sha256, 'package input ' + rel, Math.max(rec.bytes, 1));
    const data = pinned.bytes;
    const fileSha = pinned.sha256;
    if (data.length !== rec.bytes) throw new Error('package input size mismatch: ' + rel);
    const p = Buffer.from(rel, 'utf8');
    digest.update(u64(p.length));
    digest.update(p);
    digest.update(u64(data.length));
    digest.update(data);
    rows.push({ path: rel, bytes: data.length, sha256: fileSha });
  }
  const suite = digest.digest('hex');
  if (suite !== freeze.suite_digest.value || suite !== expectedDigest) throw new Error('canonical suite digest mismatch: ' + suite);
  return { package_root: root, package_file_count: found.length, suite_digest: suite, files: rows };
}
function gitContext() {
  function runGit(args) { return execFileSync(git, args, { cwd: repo, env: env, encoding: 'utf8' }); }
  const head = runGit(['rev-parse', 'HEAD']).trim();
  const indexRelative = runGit(['rev-parse', '--git-path', 'index']).trim();
  const indexPath = path.resolve(repo, indexRelative);
  const status = runGit(['status', '--porcelain=v2', '--branch', '--untracked-files=all']);
  const staged = spawnSync('/usr/bin/git', ['diff', '--cached', '--binary'], { cwd: repo, env: env, encoding: null });
  if (staged.status !== 0) throw new Error('git diff --cached failed');
  return { head: head, index_path: indexPath, index_sha256: shaFile(indexPath), cached_diff_sha256: sha(staged.stdout), status_porcelain_v2: status };
}
function commandVersion(binary, arg) {
  const r = spawnSync(binary, [arg], { encoding: 'utf8', env: env });
  if (r.status !== 0) throw new Error(binary + ' version failed: ' + r.stderr);
  return String(r.stdout).trim();
}
function commandOutput(binary, args) {
  const r = spawnSync(binary, args, {encoding:'utf8', env:env});
  if (r.status !== 0) throw new Error(binary + ' query failed: ' + r.stderr);
  return String(r.stdout || '') + String(r.stderr || '');
}
function toolContext() {
  const paths = { cargo: cargo, rustc: rustc, rustdoc: rustdoc, nextest: nextest, node: node,
    rustup: rustup, python3: python3, git: git, clang: sdkToolPaths.clang, linker: sdkToolPaths.linker, xcrun: sdkToolPaths.xcrun };
  const out = {};
  for (const name of Object.keys(paths)) out[name] = { path: paths[name], sha256: shaFile(paths[name]) };
  for (const [name, expected] of Object.entries(toolHashes)) {
    if (!out[name] || out[name].sha256 !== expected) throw new Error('pinned tool hash mismatch: ' + name);
  }
  out.cargo.version = commandVersion(cargo, '--version');
  out.rustc.version = commandVersion(rustc, '--version');
  out.rustdoc.version = commandVersion(rustdoc, '--version');
  out.nextest.version = commandVersion(nextest, '--version');
  out.node.version = commandVersion(node, '--version');
  out.rustup.version = commandVersion(rustup, '--version');
  out.python3.version = commandVersion(python3, '--version');
  out.git.version = commandVersion(git, '--version');
  out.clang.version = commandVersion(sdkToolPaths.clang, '--version');
  out.linker.version = commandOutput(sdkToolPaths.linker, ['-v']).trim();
  out.xcrun.version = commandVersion(sdkToolPaths.xcrun, '--version');
  const rustcDetails = execFileSync(rustc, ['-vV'], {encoding:'utf8', env:env});
  out.rustc.host = (rustcDetails.match(/^host: (.+)$/m) || [])[1] || null;
  out.rustc.llvm_version = (rustcDetails.match(/^LLVM version: (.+)$/m) || [])[1] || null;
  if (!out.cargo.version.startsWith('cargo 1.98.1 (797e8a9bc 2026-08-05)')
      || !out.rustc.version.startsWith('rustc 1.98.1 (48a229cea 2026-09-01)')
      || !out.rustdoc.version.startsWith('rustdoc 1.98.1 (48a229cea 2026-09-01)')
      || !out.rustc.host || out.rustc.host !== 'aarch64-apple-darwin'
      || !out.nextest.version.startsWith('cargo-nextest 0.9.146 (8af696ddc 2026-09-21)')
      || !out.node.version.startsWith('v24.20.0')
      || !out.rustup.version.startsWith('rustup 1.29.1 (d95a37b6a 2026-08-13)')
      || !out.python3.version.startsWith('Python 3.9.6')
      || !out.git.version.startsWith('git version 2.54.0 (Apple Git-157)')
      || !out.clang.version.startsWith('Apple clang version 21.0.0 (clang-2100.3.34.2)')
      || !out.linker.version.startsWith('@(#)PROGRAM:ld PROJECT:ld-27037.1')
      || !out.xcrun.version.startsWith('xcrun version 72.')) throw new Error('pinned runtime tool version changed');
  return out;
}
function linkInfo(p) {
  const st = fs.lstatSync(p);
  return { path: p, is_symlink: st.isSymbolicLink(), link_target: st.isSymbolicLink() ? fs.readlinkSync(p) : null, realpath: fs.realpathSync(p) };
}
function maybeLstat(p) {
  try { return fs.lstatSync(p); } catch (e) { if (e && e.code === 'ENOENT') return null; throw e; }
}
function readPinnedBytes(file, expectedSha, label, maximum = 64 * 1024 * 1024) {
  const nofollow = fs.constants.O_NOFOLLOW || 0;
  const fd = fs.openSync(file, fs.constants.O_RDONLY | nofollow);
  try {
    const before = fs.fstatSync(fd);
    if (!before.isFile() || before.size > maximum) throw new Error(label + ' must be a bounded regular file');
    const chunks = [];
    let total = 0;
    const buffer = Buffer.alloc(1024 * 1024);
    for (;;) {
      const count = fs.readSync(fd, buffer, 0, buffer.length, null);
      if (count === 0) break;
      total += count;
      if (total > maximum) throw new Error(label + ' exceeded the bounded read limit');
      chunks.push(Buffer.from(buffer.subarray(0, count)));
    }
    const after = fs.fstatSync(fd);
    if (before.dev !== after.dev || before.ino !== after.ino || before.size !== after.size || total !== before.size) {
      throw new Error(label + ' changed while it was read');
    }
    const bytes = Buffer.concat(chunks);
    const actualSha = sha(bytes);
    if (expectedSha && actualSha !== expectedSha) throw new Error(label + ' SHA-256 mismatch: ' + actualSha);
    return {bytes: bytes, stat: after, sha256: actualSha};
  } finally {
    fs.closeSync(fd);
  }
}
function readPinnedJson(file, expectedSha, label) {
  return JSON.parse(readPinnedBytes(file, expectedSha, label).bytes.toString('utf8'));
}
function tagContext() {
  const receipt = readPinnedJson(tagReceiptPath, tagReceiptSha256, 'tag builder receipt');
  const run = readPinnedJson(tagRunPath, tagRunSha256, 'tag builder run');
  const closure = readPinnedJson(tagClosurePath, tagClosureSha256, 'tag source closure seal');
  const executable = readPinnedBytes(tagBinary, tagBinarySha256, 'immutable tag executable', 128 * 1024 * 1024);
  const sourceRootStat = fs.lstatSync(tagSourceRoot);
  const buildRootStat = fs.lstatSync(tagBuildRoot);
  if (!sourceRootStat.isDirectory() || sourceRootStat.isSymbolicLink() || (sourceRootStat.mode & 0o222) !== 0
      || !buildRootStat.isDirectory() || buildRootStat.isSymbolicLink()
      || fs.realpathSync(tagSourceRoot) !== tagSourceRoot || fs.realpathSync(tagBuildRoot) !== tagBuildRoot
      || fs.realpathSync(tagBinary) !== tagBinary || (executable.stat.mode & 0o111) === 0) {
    throw new Error('immutable-tag source/build roots or executable permissions changed');
  }
  if (receipt.schema !== 'termrock-spec/visual-tag-holla-build-evidence-v1' || receipt.run_id !== tagRunId
      || receipt.builder_receipt_sha256 !== tagNestedReceiptSha256 || receipt.build_result !== 'PASS'
      || receipt.capture_status !== 'NOT_RUN' || receipt.admission_status !== 'NOT_RUN'
      || receipt.qualification.status !== 'blocked' || receipt.oracle_lineage.tag_object !== tagObject
      || receipt.oracle_lineage.tag_commit !== tagCommit || receipt.executable.path !== tagBinary
      || receipt.executable.sha256 !== tagBinarySha256) throw new Error('tag builder receipt identity/status changed');
  if (run.schema !== 'termrock-spec/visual-tag-holla-build-run-v1' || run.run_id !== tagRunId
      || run.state !== 'complete' || run.build_result !== 'PASS' || run.capture_status !== 'NOT_RUN'
      || run.admission_status !== 'NOT_RUN' || run.tag_builder_receipt_sha256 !== tagReceiptSha256
      || run.executable_path !== tagBinary || run.executable_sha256 !== tagBinarySha256
      || run.source_snapshot.source_commit !== tagCommit || run.source_snapshot.tree_oid !== tagTree
      || run.source_snapshot.included_file_count !== 1415) throw new Error('tag builder run identity/status changed');
  if (closure.schema !== 'termrock-vis06-tag-holla-actual-evidence-seal-v1' || closure.run_id !== tagRunId
      || closure.tag_lineage.ref !== 'refs/tags/visual-baseline'
      || closure.tag_lineage.annotated_tag_object !== tagObject || closure.tag_lineage.peeled_commit !== tagCommit
      || closure.tag_lineage.tree_oid !== tagTree || closure.source_blob_recheck.mismatches !== 0
      || closure.source_blob_recheck.included !== 1415 || closure.source_blob_recheck.source_root_read_only !== true
      || closure.executable.path !== tagBinary || closure.executable.sha256 !== tagBinarySha256
      || closure.outer_tag_builder_receipt.path !== tagReceiptPath || closure.outer_tag_builder_receipt.sha256 !== tagReceiptSha256
      || closure.nested_builder_receipt.sha256 !== tagNestedReceiptSha256
      || closure.statuses.runner !== 'PASS' || closure.statuses.selected_test !== 'termrock-visibility-tests::subjects$supervises_real_frozen_tag_holla_build'
      || closure.statuses.selected_test_passes !== 1 || closure.statuses.build !== 'PASS'
      || closure.statuses.capture !== 'NOT_RUN' || closure.statuses.admission !== 'NOT_RUN'
      || closure.statuses.qualification !== 'blocked') throw new Error('tag closure seal identity/status changed');
  const annotated = execFileSync('/usr/bin/git', ['--no-replace-objects', 'rev-parse', 'refs/tags/visual-baseline^{tag}'], {
    cwd: repo, env: {PATH: '/usr/bin:/bin', LC_ALL: 'C', GIT_NO_REPLACE_OBJECTS: '1', GIT_NO_LAZY_FETCH: '1'}, encoding: 'utf8'
  }).trim();
  const peeled = execFileSync('/usr/bin/git', ['--no-replace-objects', 'rev-parse', 'refs/tags/visual-baseline^{}'], {
    cwd: repo, env: {PATH: '/usr/bin:/bin', LC_ALL: 'C', GIT_NO_REPLACE_OBJECTS: '1', GIT_NO_LAZY_FETCH: '1'}, encoding: 'utf8'
  }).trim();
  if (annotated !== tagObject || peeled !== tagCommit) throw new Error('immutable tag reference changed');
  return {receipt_path: tagReceiptPath, receipt_sha256: tagReceiptSha256, run_path: tagRunPath, run_sha256: tagRunSha256,
    closure_path: tagClosurePath, closure_sha256: tagClosureSha256, tag_object: annotated, tag_commit: peeled,
    tag_tree: tagTree, executable_path: tagBinary, executable_bytes: executable.stat.size,
    executable_sha256: executable.sha256, source_root: fs.realpathSync(tagSourceRoot), build_root: fs.realpathSync(tagBuildRoot)};
}
function validatorContext() {
  const helper = readPinnedBytes(validatorHelper, validatorHelperSha256, 'pinned validator helper', 8 * 1024 * 1024);
  const publication = readPinnedJson(validatorSourcePublicationPath, validatorSourcePublicationSha256, 'validator source publication');
  const toolchain = readPinnedJson(validatorToolchainPath, validatorToolchainSha256, 'validator toolchain manifest');
  const sourceGitDir = '/private/tmp/termrock-vis06-r9-source-proposal-q44-64af073-20261009/private.git';
  const committed = execFileSync(git, ['--no-replace-objects', '--git-dir=' + sourceGitDir,
    'show', validatorSourceCommit + ':tools/visibility/subjects.py'], {encoding: null, env: env});
  if (sha(committed) !== validatorHelperSha256 || !committed.equals(helper.bytes)) throw new Error('validator helper copy differs from the reviewed committed Git blob');
  if (publication.schema !== 'termrock-root-publication/v1' || publication.commit !== validatorSourceCommit
      || publication.parent !== '64af07303e6a110b2f075898d6122d6edbe36db3'
      || publication.tree !== 'ff0cc9d2a51e5a54cdc54043c8177e54592a7d4d'
      || publication.push.exit_code !== 0 || publication.readback.commit !== validatorSourceCommit
      || publication.readback.exit_code !== 0 || publication.source_review_sha256 !== '4c6d5c6d99f58cd55e796fb4241c0a2a47eef19bf059c5fc790496c2a9c7e2d9'
      || publication.postcommit_review_sha256 !== '748cc7b180defba309c20d0022628086d14f8ff4659137e59de5e2429f4359d5') throw new Error('validator helper publication record does not bind the reviewed commit');
  if (toolchain.schema !== 'termrock-spec/visual-tag-validator-toolchain-v1'
      || toolchain.python.path !== '/usr/bin/python3' || toolchain.python.version !== 'Python 3.9.6'
      || toolchain.python.sha256 !== toolHashes.python3
      || toolchain.git.path !== '/usr/bin/git' || toolchain.git.version !== 'git version 2.54.0 (Apple Git-157)'
      || toolchain.git.sha256 !== toolHashes.git) throw new Error('validator toolchain manifest changed');
  return {helper_path: validatorHelper, helper_sha256: helper.sha256, helper_bytes: helper.stat.size,
    source_commit: validatorSourceCommit, source_publication_path: validatorSourcePublicationPath,
    source_publication_sha256: validatorSourcePublicationSha256, toolchain_path: validatorToolchainPath,
    toolchain_sha256: validatorToolchainSha256};
}
function rendererContext() {
  const rootStat = fs.lstatSync(rendererRoot);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink() || fs.realpathSync(rendererRoot) !== rendererRoot) {
    throw new Error('locked Tuiscotti renderer source root must be a canonical real directory');
  }
  const lockBytes = readPinnedBytes(path.join(packageRoot, 'Cargo.lock'), expectedLockSha256, 'locked renderer dependency closure');
  const source = 'source = "git+https://github.com/tailrocks/tuiscotti?rev=' + rendererCommit + '#' + rendererCommit + '"';
  const blocks = lockBytes.bytes.toString('utf8').split('[[package]]');
  for (const name of ['tuiscotti', 'tuiscotti-render']) {
    const block = blocks.find(row => row.includes('name = "' + name + '"'));
    if (!block || !block.includes(source)) throw new Error('Cargo.lock does not bind ' + name + ' to the reviewed Tuiscotti commit');
  }
  const files = [];
  for (const [relative, expected] of Object.entries(rendererFiles)) {
    const pinned = readPinnedBytes(path.join(rendererRoot, relative), expected, 'locked renderer input ' + relative, 16 * 1024 * 1024);
    files.push({path:relative, bytes:pinned.stat.size, sha256:pinned.sha256});
  }
  const profilePin = freezePath;
  const profileRecord = JSON.parse(readPinnedBytes(profilePin, expectedFreezeSha, 'candidate package freeze').bytes.toString('utf8'))
    .files.find(row => row.path === 'profile.json');
  const profileBytes = readPinnedBytes(path.join(packageRoot, 'profile.json'), profileRecord.sha256, 'vendored renderer profile');
  const profile = JSON.parse(profileBytes.bytes.toString('utf8'));
  if (profile.tool !== 'tuiscotti' || profile.tool_revision !== rendererCommit
      || profile.id !== 'tuiscotti-default-parity-v1' || profile.screen_profile !== 'default'
      || profile.renderer.id !== 'tuiscotti-default' || profile.renderer.implementation !== 'strict-vendored-profile'
      || profile.renderer.expected_hash !== expectedSuite.rendererHash || profile.renderer.renderer_version !== 1
      || profile.terminal.term !== 'xterm-256color' || profile.terminal.colorterm !== 'truecolor'
      || profile.environment.clear !== true || JSON.stringify(profile.environment.case_allowlist) !== '["HOLLA_NO_HISTORY"]') {
    throw new Error('suite profile does not match the locked vendored-renderer contract');
  }
  return {source_root:rendererRoot, source_root_realpath:fs.realpathSync(rendererRoot), source_commit:rendererCommit,
    package_lock_sha256:expectedLockSha256, profile_sha256:profileBytes.sha256,
    profile:{id:profile.id, tool_revision:profile.tool_revision, renderer:profile.renderer,
      terminal:profile.terminal, environment:profile.environment, visual_acceptance:profile.visual_acceptance}, files:files};
}
function sdkContext() {
  if (process.platform !== 'darwin' || process.arch !== 'arm64') throw new Error('tag capture runner is pinned to Darwin arm64');
  const developerStat = fs.lstatSync(developerDir);
  if (!developerStat.isDirectory() || developerStat.isSymbolicLink() || fs.realpathSync(developerDir) !== developerDir) {
    throw new Error('Command Line Tools root must be a canonical real directory');
  }
  const sdkStat = fs.lstatSync(sdkRoot);
  if (!sdkStat.isSymbolicLink() || fs.readlinkSync(sdkRoot) !== 'MacOSX27.0.sdk'
      || fs.realpathSync(sdkRoot) !== sdkResolvedRoot || env.DEVELOPER_DIR !== developerDir || env.SDKROOT !== sdkRoot) {
    throw new Error('Command Line Tools SDK selection changed');
  }
  const settingsJson = readPinnedBytes(sdkSettingsJson, sdkHashes.sdk_settings_json, 'macOS SDK settings JSON', 8 * 1024 * 1024);
  const settingsPlist = readPinnedBytes(sdkSettingsPlist, sdkHashes.sdk_settings_plist, 'macOS SDK settings plist', 8 * 1024 * 1024);
  const settings = JSON.parse(settingsJson.bytes.toString('utf8'));
  const selectedSdk = execFileSync(sdkToolPaths.xcrun, ['--show-sdk-path'], {encoding:'utf8', env:env}).trim();
  const sdkVersion = execFileSync(sdkToolPaths.xcrun, ['--show-sdk-version'], {encoding:'utf8', env:env}).trim();
  const sdkBuildVersion = execFileSync(sdkToolPaths.xcrun, ['--show-sdk-build-version'], {encoding:'utf8', env:env}).trim();
  const selectedClang = execFileSync(sdkToolPaths.xcrun, ['--find', 'clang'], {encoding:'utf8', env:env}).trim();
  const selectedLinker = execFileSync(sdkToolPaths.xcrun, ['--find', 'ld'], {encoding:'utf8', env:env}).trim();
  const selectedDeveloperDir = execFileSync('/usr/bin/xcode-select', ['-p'], {encoding:'utf8', env:env}).trim();
  const osVersion = execFileSync('/usr/bin/sw_vers', ['-productVersion'], {encoding:'utf8', env:env}).trim();
  if (selectedSdk !== sdkRoot || sdkVersion !== '27.0' || sdkBuildVersion !== '26A425'
      || settings.Version !== sdkVersion || selectedClang !== sdkToolPaths.clang
      || selectedLinker !== sdkToolPaths.linker || selectedDeveloperDir !== developerDir || osVersion !== '27.0.1') {
    throw new Error('macOS SDK or Command Line Tools selection differs from the reviewed host facts');
  }
  return {platform:process.platform, architecture:process.arch, macos_version:osVersion,
    developer_dir:developerDir, developer_dir_realpath:fs.realpathSync(developerDir),
    sdk_root:sdkRoot, sdk_root_link_target:fs.readlinkSync(sdkRoot), sdk_resolved_root:sdkResolvedRoot,
    sdk_version:sdkVersion, sdk_build_version:sdkBuildVersion, sdk_settings_json_sha256:settingsJson.sha256,
    sdk_settings_plist_sha256:settingsPlist.sha256, xcrun_path:sdkToolPaths.xcrun,
    xcrun_sha256:shaFile(sdkToolPaths.xcrun), selected_clang:selectedClang, selected_linker:selectedLinker,
    clang_sha256:shaFile(selectedClang), linker_sha256:shaFile(selectedLinker), rust_host:'aarch64-apple-darwin'};
}
function reviewContext() {
  const source = readPinnedJson(sourceReviewPath, sourceReviewSha256, 'R2 baseline source review');
  const candidateSource = readPinnedJson(candidateSourceReviewPath, candidateSourceReviewSha256, 'd990 candidate source review');
  const contractReview = readPinnedBytes(contractReviewPath, contractReviewSha256, 'R2 contract gate review');
  const contractReceipt = readPinnedBytes(contractReceiptPath, contractReceiptSha256, 'R2 contract gate receipt');
  const runnerReview = readPinnedBytes(runnerReviewPath, runnerReviewSha256, 'R2 template runner review');
  if (source.verdict !== 'READY' || source.subject !== 'VIS-02 Showcase initial checkpoint and contract regression'
      || source.candidate.files['src/lib.rs'] !== '9710e5d2cf3cdd9acad9e50cd68841613fd7c25a5751f93f4955bfe450cc509a') {
    throw new Error('R2 source review baseline binding changed');
  }
  const reviewed = candidateSource.reviewed_artifacts || {};
  const basePackage = reviewed.base_package || {};
  const candidatePackage = reviewed.candidate_package || {};
  const patch = reviewed.patch || {};
  const capturePlan = reviewed.capture_plan || {};
  const helper = reviewed.helper_bytes || {};
  if (candidateSource.schema !== 'termrock-vis02-helper-pin-source-review/v1'
      || candidateSource.reviewer !== '/root/controls_wrapper_review_luna'
      || candidateSource.verdict !== 'READY_SOURCE_DELTA_ONLY' || candidateSource.findings.length !== 0
      || basePackage.path !== prefetchPackageRoot
      || basePackage.suite_sha256 !== '4a2bd6845c5e310b8964c100e9c6deaae395a248e3ed359f4d38aa8d5c800d66'
      || basePackage.cargo_lock_sha256 !== expectedLockSha256
      || candidatePackage.path !== '/private/tmp/termrock-vis02-helperpin-d990-candidate-20261009-q47'
      || candidatePackage.package_file_count !== 28 || candidatePackage.recomputed_suite_sha256 !== expectedDigest
      || candidatePackage.src_lib_sha256 !== '7830f8a7069eb622fbbd4126ddcf42dd658e4dff23c108d76058e7cf986df3f7'
      || candidatePackage.cargo_lock_sha256 !== expectedLockSha256
      || candidatePackage.package_freeze_sha256 !== '9b4ac40fdfc28e2590c12bdd4bf115c490f7296166458cd455d3b48d2a9416e0'
      || patch.path !== '/private/tmp/termrock-vis02-helperpin-d990-candidate-20261009-q47.patch'
      || patch.sha256 !== 'dd3325f9ae17c0e4877953f2c072caaf4782553969ed113c93bd140c6f9da9bd'
      || capturePlan.path !== '/private/tmp/termrock-vis02-helperpin-d990-runner-plan-20261009-q47.md'
      || capturePlan.sha256 !== '3430b360333131241541fbddb497a06c442a9d12e3b01979d5c06125a9f5629e'
      || helper.path !== validatorHelper || helper.sha256 !== validatorHelperSha256) {
    throw new Error('d990 candidate source review does not bind this exact helper-pin successor');
  }
  return {r2_baseline_source_review:{path:sourceReviewPath,sha256:sourceReviewSha256,helper_pin:'0e09a25178749178252846512a66ad1dd1787ac9f27c96b5251e5a3aa61680c3'},
    d990_candidate_source_review:{path:candidateSourceReviewPath,sha256:candidateSourceReviewSha256,reviewer:candidateSource.reviewer,
      verdict:candidateSource.verdict,candidate_suite_sha256:candidatePackage.recomputed_suite_sha256,
      candidate_lib_sha256:candidatePackage.src_lib_sha256,helper_sha256:helper.sha256},
    r2_contract_review:{path:contractReviewPath,sha256:contractReview.sha256},
    r2_contract_receipt:{path:contractReceiptPath,sha256:contractReceipt.sha256},
    r2_runner_template_review:{path:runnerReviewPath,sha256:runnerReview.sha256},
    baseline_review_applies_to_d990_candidate:false};
}
function outputTree(root) {
  const rows = [];
  function walk(dir, prefix) {
    for (const ent of fs.readdirSync(dir, {withFileTypes:true})) {
      const rel = prefix ? prefix + '/' + ent.name : ent.name;
      const full = path.join(dir, ent.name);
      const st = fs.lstatSync(full);
      if (st.isSymbolicLink()) throw new Error('captured output contains a symlink: ' + rel);
      if (st.isDirectory()) walk(full, rel);
      else if (st.isFile()) rows.push({path:full, relative:rel, bytes:st.size, sha256:shaFile(full), device:String(st.dev), inode:String(st.ino), links:st.nlink});
      else throw new Error('captured output contains a special file: ' + rel);
    }
  }
  walk(root, '');
  rows.sort((a,b)=>Buffer.compare(Buffer.from(a.relative),Buffer.from(b.relative)));
  if (rows.some(row => row.links !== 1) || new Set(rows.map(row=>row.device + ':' + row.inode)).size !== rows.length) throw new Error('captured outputs must have unique physical files');
  return rows;
}
function captureContext() {
  const write = fs.lstatSync(actualRoot);
  const output = fs.lstatSync(actualOutputRoot);
  if (!write.isDirectory() || write.isSymbolicLink() || !output.isDirectory() || output.isSymbolicLink()) throw new Error('capture roots must be real directories');
  const receiptStat = maybeLstat(captureReceiptPath);
  if (receiptStat && (!receiptStat.isFile() || receiptStat.isSymbolicLink() || receiptStat.nlink !== 1)) throw new Error('capture receipt must be a unique regular file');
  const outputs = outputTree(actualOutputRoot);
  if (!receiptStat) return {write_root:actualRoot, actual_output_root:actualOutputRoot, receipt_exists:false, output_files:outputs};
  const receiptBytes = readPinnedBytes(captureReceiptPath, null, 'capture receipt', 32 * 1024 * 1024);
  const receipt = JSON.parse(receiptBytes.bytes.toString('utf8'));
  const interactions = receipt.checks.filter(check => check.dimension === 'interaction' && check.subject_role === 'oracle' && check.status === 'PASS');
  const checkpointIds = receipt.case.checkpoints.map(row=>row.id);
  const formats = ['frame_json','ansi','html','png','ascii','txt','ascii_loss_json','png_fidelity_json','observations_json','manifest_json'];
  const artifactKeys = new Set();
  const artifactPaths = new Set();
  for (const artifact of receipt.artifacts) {
    const relative = path.relative(actualOutputRoot, artifact.path);
    const key = artifact.checkpoint_id + ':' + artifact.format;
    if (!checkpointIds.includes(artifact.checkpoint_id) || !formats.includes(artifact.format)
        || artifactKeys.has(key) || artifactPaths.has(artifact.path) || !relative
        || relative.startsWith('..' + path.sep) || path.isAbsolute(relative) || path.resolve(artifact.path) !== artifact.path) {
      throw new Error('capture receipt contains an invalid or duplicate artifact binding');
    }
    artifactKeys.add(key);
    artifactPaths.add(artifact.path);
  }
  if (receipt.schema !== 'termrock-spec/parity-oracle-capture-receipt-v1' || receipt.run_id !== tagRunId
      || receipt.case.id !== 'HELP-HOLLA-004' || receipt.capture_status !== 'COMPLETE' || receipt.state !== 'capture_recorded'
      || receipt.admission_status !== 'NOT_RUN' || receipt.execution_anchor_status !== 'unverified'
      || receipt.qualification.status !== 'blocked' || receipt.suite.digest !== expectedDigest
      || receipt.suite.compiled_digest !== expectedDigest || receipt.suite.dependency_lock_sha256 !== expectedLockSha256
      || receipt.suite.revision !== expectedSuite.revision || receipt.suite.case_set_digest !== expectedSuite.caseSetDigest
      || receipt.suite.profile_digest !== expectedSuite.profileDigest || receipt.renderer.hash !== expectedSuite.rendererHash
      || receipt.case.geometry.cols !== 120 || receipt.case.geometry.rows !== 40
      || receipt.oracle_identity.tag_object !== tagObject || receipt.oracle_identity.tag_commit !== tagCommit
      || receipt.oracle_identity.source_snapshot_tree !== tagTree
      || receipt.oracle_identity.builder_receipt_sha256 !== tagNestedReceiptSha256
      || receipt.oracle_identity.executable.expected_sha256 !== tagBinarySha256
      || receipt.oracle_identity.executable.actual_sha256 !== tagBinarySha256
      || receipt.builder_evidence.path !== tagReceiptPath || receipt.builder_evidence.sha256 !== tagReceiptSha256
      || receipt.builder_run.path !== tagRunPath || receipt.builder_run.sha256 !== tagRunSha256
      || receipt.validator_helper.path !== validatorHelper
      || receipt.validator_helper.sha256 !== validatorHelperSha256 || receipt.artifacts.length !== 40
      || checkpointIds.length !== 4 || new Set(checkpointIds).size !== 4 || interactions.length !== 12
      || receipt.checks.some(check => check.dimension === 'visual' && check.status === 'PASS')
      || receipt.write_policy.write_root !== actualRoot || receipt.write_policy.receipt_path !== captureReceiptPath) {
    throw new Error('capture receipt does not satisfy the immutable-tag actual-only contract');
  }
  if (artifactKeys.size !== 40 || outputs.length !== 40 || receipt.artifacts.some(artifact => {
    const row = outputs.find(item => item.path === artifact.path);
    return !row || row.bytes !== artifact.bytes || row.sha256 !== artifact.sha256;
  })) throw new Error('captured output files do not match the receipt artifact inventory');
  return {write_root:actualRoot, actual_output_root:actualOutputRoot, receipt_exists:true,
    receipt_sha256:receiptBytes.sha256, receipt_bytes:receiptBytes.stat.size, artifact_count:outputs.length,
    interaction_assertions:interactions.length, checkpoint_ids:checkpointIds,
    capture_status:receipt.capture_status, execution_anchor_status:receipt.execution_anchor_status,
    qualification_status:receipt.qualification.status, admission_status:receipt.admission_status, output_files:outputs};
}
function cacheContext() {
  const homeStat = fs.lstatSync(cargoHome);
  const registryPath = path.join(cargoHome, 'registry');
  const gitPath = path.join(cargoHome, 'git');
  const registryStat = fs.lstatSync(registryPath);
  const gitStat = fs.lstatSync(gitPath);
  for (const configName of ['config', 'config.toml']) {
    if (maybeLstat(path.join(cargoHome, configName))) throw new Error('private CARGO_HOME must not contain user configuration: ' + configName);
  }
  if (!homeStat.isDirectory() || homeStat.isSymbolicLink() || (homeStat.mode & 0o777) !== 0o700
      || !registryStat.isDirectory() || registryStat.isSymbolicLink() || !gitStat.isDirectory() || gitStat.isSymbolicLink()
      || fs.realpathSync(cargoHome) !== cargoHome || fs.realpathSync(registryPath) !== registryPath || fs.realpathSync(gitPath) !== gitPath) {
    throw new Error('CARGO_HOME and its registry/git roots must be canonical real directories; CARGO_HOME must be mode 0700');
  }
  const cacheProofBytes = readPinnedBytes(cacheVerificationPath, cacheVerificationSha256, 'R2 cache verification proof');
  const cacheProofSha = cacheProofBytes.sha256;
  const cacheProof = JSON.parse(cacheProofBytes.bytes.toString('utf8'));
  const lockSha = shaFile(path.join(packageRoot, 'Cargo.lock'));
  if (cacheProof.cargo_home !== cargoHome || cacheProof.lock_sha256 !== expectedLockSha256 || lockSha !== expectedLockSha256 || cacheProof.registry_lock_package_count !== 206 || cacheProof.verified_registry_archive_count !== 206 || cacheProof.verified_registry_index_count !== 206 || cacheProof.lock_git_package_count !== 5 || cacheProof.verified_git_checkout_count !== 5 || cacheProof.failures.length !== 0) throw new Error('cache proof does not bind the frozen Cargo.lock closure');
  const prefetchBytes = readPinnedBytes(prefetchCompletionPath, prefetchCompletionSha256, 'R2 prefetch completion proof');
  const prefetchSha = prefetchBytes.sha256;
  const prefetch = JSON.parse(prefetchBytes.bytes.toString('utf8'));
  if (prefetch.status !== 'PREFETCH_AND_ALL_FEATURES_OFFLINE_METADATA_SUCCEEDED_NOT_TESTED' || prefetch.candidate.root !== prefetchPackageRoot || prefetch.candidate.lock_sha256 !== expectedLockSha256 || prefetch.cache_verification.sha256 !== cacheVerificationSha256 || prefetch.all_features_metadata.exit_code !== 0 || prefetch.prefetch_offline_verification.exit_code !== 0) throw new Error('prefetch receipt does not bind the unchanged locked dependency closure');
  const targetStat = maybeLstat(targetDir);
  if (!targetStat || !targetStat.isDirectory() || targetStat.isSymbolicLink() || fs.realpathSync(targetDir) !== targetDir) throw new Error('reused target dir must be a canonical real directory');
  const testBinaryRoot = path.join(targetDir, 'debug/deps');
  const testBinaries = fs.readdirSync(testBinaryRoot).filter(name => /^holla_tag_capture-[0-9a-f]+$/.test(name));
  if (testBinaries.length !== 1 || testBinaries[0] !== captureTestBinaryName) throw new Error('reused target must contain only the previously built pinned capture test binary');
  const captureBinary = readPinnedBytes(captureTestBinaryPath, captureTestBinarySha256, 'previously built capture test binary', 128 * 1024 * 1024);
  return { cargo_home: cargoHome, cargo_home_realpath: fs.realpathSync(cargoHome), registry: linkInfo(registryPath), git: linkInfo(gitPath), cache_verification_path: cacheVerificationPath, cache_verification_sha256: cacheProofSha, registry_locked_packages: cacheProof.verified_registry_archive_count, git_locked_packages: cacheProof.verified_git_checkout_count, prefetch_completion_path: prefetchCompletionPath, prefetch_completion_sha256: prefetchSha, target_dir: targetDir, target_dir_exists: true, target_dir_realpath: fs.realpathSync(targetDir), target_dir_device: String(targetStat.dev), target_dir_inode: String(targetStat.ino), holla_tag_capture_binaries: testBinaries, holla_tag_capture_binary: {path:captureTestBinaryPath,sha256:captureBinary.sha256,bytes:captureBinary.stat.size} };
}
function snapshot(label) {
  const freezeBytes = readPinnedBytes(freezePath, expectedFreezeSha, 'candidate package freeze');
  const freeze = JSON.parse(freezeBytes.bytes.toString('utf8'));
  const freezeSha = freezeBytes.sha256;
  if (freezeSha !== expectedFreezeSha) throw new Error('freeze file hash changed: ' + freezeSha);
  const pkg = inventory(freeze);
  const runnerSha = shaFile(__filename);
  if (shaFile(runnerTemplatePath) !== runnerTemplateSha256 || shaFile(r13ReferenceRunnerPath) !== r13ReferenceRunnerSha256
      || shaFile(r7PatternReferencePath) !== r7PatternReferenceSha256) throw new Error('frozen runner source/reference changed');
  return { label: label, at_utc: new Date().toISOString(), source_root: packageRoot,
    runner: { path: __filename, sha256: runnerSha, template_path: runnerTemplatePath, template_sha256: runnerTemplateSha256,
      r7_pattern_reference_path:r7PatternReferencePath,r7_pattern_reference_sha256:r7PatternReferenceSha256,r7_pattern_reference_executed:false,
      r13_reference_path: r13ReferenceRunnerPath, r13_reference_sha256: r13ReferenceRunnerSha256, r13_executed:false },
    freeze_sha256: freezeSha, suite_digest: pkg.suite_digest, package_file_count: pkg.package_file_count,
    package_files: pkg.files, tools: toolContext(), sdk: sdkContext(), renderer: rendererContext(), reviews: reviewContext(),
    git: gitContext(), cache: cacheContext(), tag: tagContext(), validator: validatorContext(), capture: captureContext() };
}
const args = ['nextest', 'run', '--profile', 'default', '--manifest-path', path.join(packageRoot, 'Cargo.toml'), '--package', 'termrock-e2e', '--test', 'holla_tag_capture', '--locked', '--offline', '--build-jobs', '2', '--test-threads', '1', '--user-config-file', 'none', '--run-ignored', 'only', '--ignore-default-filter', '--filterset', 'package(=termrock-e2e) & binary(=holla_tag_capture) & test(=captures_holla_from_pinned_immutable_tag)', '--no-tests', 'fail', '--no-fail-fast', '--retries', '0', '--color', 'never', '--show-progress', 'none', '--success-output', 'final', '--failure-output', 'immediate-final', '--final-status-level', 'all', '--no-output-indent', '--message-format', 'libtest-json-plus', '--message-format-version', '0.1'];
const env = { HOME: path.join(runRoot, 'home'), TMPDIR: path.join(runRoot, 'tmp'), TMP: path.join(runRoot, 'tmp'), TEMP: path.join(runRoot, 'tmp'), PATH: '/Users/donbeave/.cargo/bin:/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146:/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin', CARGO: cargo, CARGO_HOME: cargoHome, CARGO_TARGET_DIR: targetDir, CARGO_BUILD_JOBS: '2', CARGO_INCREMENTAL: '0', CARGO_NET_OFFLINE: 'true', CARGO_TERM_COLOR: 'never', NEXTEST_EXPERIMENTAL_LIBTEST_JSON: '1', RUSTUP_HOME: '/Users/donbeave/.rustup', RUSTUP_TOOLCHAIN: '1.98.1', RUSTC: rustc, RUSTDOC: rustdoc, DEVELOPER_DIR: developerDir, SDKROOT: sdkRoot, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_OPTIONAL_LOCKS: '0', GIT_NO_REPLACE_OBJECTS: '1', GIT_NO_LAZY_FETCH: '1', CI: '1', LANG: 'C', LC_ALL: 'C.UTF-8', TERM: 'dumb', TERMROCK_E2E_ORACLE_BUILD_RECEIPT: tagReceiptPath, TERMROCK_E2E_ORACLE_BUILD_RECEIPT_SHA256: tagReceiptSha256, TERMROCK_E2E_ORACLE_ACTUAL_ROOT: actualOutputRoot, TERMROCK_VIS06_SUBJECTS_PY: validatorHelper, TERMROCK_VIS06_GIT_REPOSITORY: repo, TERMROCK_VIS06_VALIDATOR_TOOLCHAIN: validatorToolchainPath, TERMROCK_VIS06_VALIDATOR_TOOLCHAIN_SHA256: validatorToolchainSha256, TERMROCK_E2E_WRITE_ROOT: actualRoot, TERMROCK_E2E_RECEIPT_PATH: captureReceiptPath };
env.TERMROCK_E2E_PROTECTED_ROOTS = JSON.stringify([
  {kind:'subject_source', role:'oracle', path:tagSourceRoot},
  {kind:'subject_artifact', role:'oracle', path:tagBuildRoot},
  {kind:'oracle', role:null, path:tagSourceRoot}
]);
function readLaunchGrant() {
  if (process.argv.length !== 3 || !/^[0-9a-f]{64}$/.test(process.argv[2] || '')) throw new Error('one Root launch-grant SHA-256 argument is required');
  const grantSha = process.argv[2];
  const grant = readPinnedJson(launchGrantPath, grantSha, 'Root launch grant');
  const manifestSha = grant.invocation_manifest_sha256;
  if (typeof manifestSha !== 'string' || !/^[0-9a-f]{64}$/.test(manifestSha)) throw new Error('Root launch grant must bind an invocation-manifest SHA-256');
  const manifest = readPinnedJson(invocationManifestPath, manifestSha, 'frozen invocation manifest');
  const required = {
    schema:'termrock-rust-tool-test-execution/tag-capture-launch-grant-v3',
    decision:'RUN_ONE_IGNORED_IMMUTABLE_TAG_CAPTURE',
    attempt_id:path.basename(evidenceRoot),
    runner_sha256:shaFile(__filename),
    package_digest:expectedDigest,
    package_freeze_sha256:expectedFreezeSha,
    cargo_lock_sha256:expectedLockSha256,
    invocation_manifest_sha256:manifestSha,
    cache_verification_sha256:cacheVerificationSha256,
    prefetch_completion_sha256:prefetchCompletionSha256,
    reused_target_dir:targetDir,
    selected_target:'holla_tag_capture',
    selected_test:'captures_holla_from_pinned_immutable_tag',
    test_threads:1,
    build_jobs:2,
    retry_count:0,
    timeout_ms:timeoutMs,
    max_output_bytes: maxRawOutputBytes,
    tag_builder_receipt_sha256:tagReceiptSha256,
    tag_executable_sha256:tagBinarySha256,
    tag_object:tagObject,
    tag_commit:tagCommit,
    tag_tree:tagTree,
    validator_source_commit:validatorSourceCommit,
    validator_helper_sha256:validatorHelperSha256,
    candidate_source_review_sha256:candidateSourceReviewSha256,
    validator_source_publication_sha256:validatorSourcePublicationSha256,
    actual_root:actualRoot,
    actual_output_root:actualOutputRoot,
    receipt_path:captureReceiptPath,
    renderer_source_commit:rendererCommit,
    renderer_profile_sha256:expectedSuite.profileDigest,
    renderer_hash:expectedSuite.rendererHash,
    sdk_version:'27.0',
    sdk_build_version:'26A425',
    r2_contract_receipt_sha256:contractReceiptSha256,
    r2_contract_review_sha256:contractReviewSha256,
    r13_runner_executed:false,
    allow_admission:false,
    allow_expected_generation:false,
    allow_qualification:false,
    execution_authorized:true,
    template_status:'ROOT_AUTHORIZED_EXECUTION'
  };
  for (const [key, value] of Object.entries(required)) if (grant[key] !== value) throw new Error('Root launch grant field mismatch: ' + key);
  const manifestEnv = manifest.command && manifest.command.environment;
  const envKeys = Object.keys(env).sort();
  const manifestEnvKeys = manifestEnv && typeof manifestEnv === 'object' ? Object.keys(manifestEnv).sort() : [];
  const manifestEnvMatches = envKeys.length === manifestEnvKeys.length && envKeys.every(key => manifestEnv[key] === env[key]);
  const manifestToolsMatch = manifest.tools && Object.entries(toolHashes).every(([name, digest]) =>
    manifest.tools[name] && manifest.tools[name].sha256 === digest);
  if (manifest.schema !== 'termrock-rust-tool-test-execution/tag-capture-invocation-v1'
      || manifest.attempt_id !== path.basename(evidenceRoot)
      || manifest.command.executable !== cargo
      || JSON.stringify(manifest.command.argv) !== JSON.stringify([cargo].concat(args))
      || manifest.command.cwd !== packageRoot || !manifestEnvMatches
      || manifest.bounds.timeout_ms !== timeoutMs || manifest.bounds.raw_output_limit_bytes_total !== maxRawOutputBytes
      || manifest.bounds.build_jobs !== 2 || manifest.bounds.test_threads !== 1 || manifest.bounds.retries !== 0
      || manifest.target.package !== 'termrock-e2e' || manifest.target.binary !== 'holla_tag_capture'
      || manifest.target.test !== 'captures_holla_from_pinned_immutable_tag'
      || manifest.inputs.candidate_suite_digest !== expectedDigest || manifest.inputs.candidate_freeze_sha256 !== expectedFreezeSha
      || manifest.inputs.cargo_lock_sha256 !== expectedLockSha256 || manifest.inputs.validator_helper_sha256 !== validatorHelperSha256
      || manifest.inputs.candidate_package_root !== packageRoot || manifest.inputs.candidate_freeze_path !== freezePath
      || manifest.inputs.candidate_source_review_path !== candidateSourceReviewPath
      || manifest.inputs.candidate_source_review_sha256 !== candidateSourceReviewSha256
      || manifest.inputs.capture_test_binary_path !== captureTestBinaryPath
      || manifest.inputs.capture_test_binary_sha256 !== captureTestBinarySha256
      || manifest.inputs.validator_source_commit !== validatorSourceCommit || manifest.inputs.tag_builder_receipt_sha256 !== tagReceiptSha256
      || manifest.inputs.tag_executable_sha256 !== tagBinarySha256 || manifest.inputs.cache_verification_sha256 !== cacheVerificationSha256
      || manifest.inputs.prefetch_completion_sha256 !== prefetchCompletionSha256 || manifest.inputs.renderer_source_commit !== rendererCommit
      || manifest.inputs.renderer_profile_sha256 !== expectedSuite.profileDigest || manifest.inputs.renderer_hash !== expectedSuite.rendererHash
      || manifest.inputs.tag_nested_receipt_sha256 !== tagNestedReceiptSha256 || manifest.inputs.tag_run_sha256 !== tagRunSha256
      || manifest.inputs.tag_closure_sha256 !== tagClosureSha256 || manifest.inputs.tag_object !== tagObject
      || manifest.inputs.tag_commit !== tagCommit || manifest.inputs.tag_tree !== tagTree
      || manifest.inputs.validator_source_publication_sha256 !== validatorSourcePublicationSha256
      || manifest.inputs.validator_toolchain_sha256 !== validatorToolchainSha256
      || manifest.inputs.validator_source_publication_path !== validatorSourcePublicationPath
      || manifest.inputs.validator_toolchain_path !== validatorToolchainPath
      || manifest.inputs.tag_builder_receipt_path !== tagReceiptPath || manifest.inputs.tag_run_path !== tagRunPath
      || manifest.inputs.tag_closure_path !== tagClosurePath || manifest.inputs.tag_executable_path !== tagBinary
      || manifest.inputs.cache_verification_path !== cacheVerificationPath
      || manifest.inputs.prefetch_completion_path !== prefetchCompletionPath
      || manifest.inputs.cargo_home !== cargoHome || manifest.inputs.target_dir !== targetDir
      || manifest.inputs.suite_revision !== expectedSuite.revision
      || manifest.inputs.case_set_digest !== expectedSuite.caseSetDigest
      || manifest.inputs.profile_digest !== expectedSuite.profileDigest
      || manifest.inputs.renderer_hash !== expectedSuite.rendererHash
      || manifest.outputs.write_root !== actualRoot || manifest.outputs.actual_output_root !== actualOutputRoot
      || manifest.outputs.receipt_path !== captureReceiptPath || manifest.policy.expected_generation !== false
      || manifest.policy.admission !== false || manifest.policy.qualification !== false || manifest.policy.r13_runner_executed !== false) {
    throw new Error('frozen invocation manifest differs from the actual runner command or bounded evidence contract');
  }
  if (manifest.host.platform !== 'darwin' || manifest.host.architecture !== 'arm64'
      || manifest.host.macos_version !== '27.0.1' || manifest.host.developer_dir !== developerDir
      || manifest.host.sdk_root !== sdkRoot || manifest.host.sdk_resolved_root !== sdkResolvedRoot
      || manifest.host.sdk_version !== '27.0' || manifest.host.sdk_build_version !== '26A425'
      || manifest.host.clang_sha256 !== toolHashes.clang || manifest.host.linker_sha256 !== toolHashes.linker
      || manifest.host.xcrun_sha256 !== toolHashes.xcrun
      || manifest.renderer.source_root !== rendererRoot || manifest.renderer.source_commit !== rendererCommit
      || manifest.renderer.profile_sha256 !== expectedSuite.profileDigest || manifest.renderer.renderer_hash !== expectedSuite.rendererHash
      || manifest.renderer.profile_id !== 'tuiscotti-default-parity-v1'
      || manifest.renderer.tool_revision !== rendererCommit || manifest.renderer.implementation !== 'strict-vendored-profile'
      || JSON.stringify(manifest.renderer.files) !== JSON.stringify(Object.entries(rendererFiles).map(([file, digest]) => ({path:file,sha256:digest})))
      || manifest.baseline_evidence.r2_source_review_sha256 !== sourceReviewSha256
      || manifest.baseline_evidence.r2_contract_review_sha256 !== contractReviewSha256
      || manifest.baseline_evidence.r2_contract_receipt_sha256 !== contractReceiptSha256
      || manifest.baseline_evidence.r2_runner_review_sha256 !== runnerReviewSha256
      || manifest.baseline_evidence.source_review_applies_to_d990_candidate !== false
      || manifest.candidate_evidence.d990_source_review_path !== candidateSourceReviewPath
      || manifest.candidate_evidence.d990_source_review_sha256 !== candidateSourceReviewSha256
      || manifest.candidate_evidence.candidate_suite_digest !== expectedDigest
      || manifest.candidate_evidence.executed_package_root !== packageRoot
      || manifest.candidate_evidence.candidate_lib_sha256 !== '7830f8a7069eb622fbbd4126ddcf42dd658e4dff23c108d76058e7cf986df3f7'
      || manifest.candidate_evidence.helper_sha256 !== validatorHelperSha256
      || manifest.r13_reference.path !== r13ReferenceRunnerPath || manifest.r13_reference.sha256 !== r13ReferenceRunnerSha256
      || manifest.r13_reference.executed !== false
      || manifest.r2_runner_template.path !== runnerTemplatePath || manifest.r2_runner_template.sha256 !== runnerTemplateSha256
      || manifest.r2_runner_template.invoked_by_this_run !== false
      || manifest.r7_pattern_reference.path !== r7PatternReferencePath
      || manifest.r7_pattern_reference.sha256 !== r7PatternReferenceSha256
      || manifest.r7_pattern_reference.executed !== false
      || manifest.policy.test_authoring !== 'Rust Nextest integration test; invokes the production Python CLI only as a subprocess black box') {
    throw new Error('manifest host, renderer, or historical review/reference binding changed');
  }
  if (!manifestToolsMatch || manifest.host.sdk_settings_json_sha256 !== sdkHashes.sdk_settings_json
      || manifest.host.sdk_settings_plist_sha256 !== sdkHashes.sdk_settings_plist
      || manifest.host.rust_host !== 'aarch64-apple-darwin'
      || manifest.pin_successor.base_package_root !== prefetchPackageRoot
      || manifest.pin_successor.candidate_package_root !== packageRoot
      || manifest.pin_successor.source_candidate_path !== '/private/tmp/termrock-vis02-helperpin-d990-candidate-20261009-q47'
      || manifest.pin_successor.plan_path !== '/private/tmp/termrock-vis02-helperpin-d990-runner-plan-20261009-q47.md'
      || manifest.pin_successor.plan_sha256 !== '3430b360333131241541fbddb497a06c442a9d12e3b01979d5c06125a9f5629e'
      || manifest.pin_successor.previous_helper_sha256 !== '0e09a25178749178252846512a66ad1dd1787ac9f27c96b5251e5a3aa61680c3'
      || manifest.pin_successor.patch_sha256 !== 'dd3325f9ae17c0e4877953f2c072caaf4782553969ed113c93bd140c6f9da9bd'
      || manifest.pin_successor.new_helper_sha256 !== validatorHelperSha256
      || manifest.pin_successor.changed_path !== 'src/lib.rs'
      || manifest.launch.required !== true || manifest.launch.authorized !== false
      || manifest.launch.launch_grant_path !== launchGrantPath || manifest.launch.launch_grant_sha256_argument_required !== true) {
    throw new Error('manifest toolchain, SDK settings, or one-pin source successor binding changed');
  }
  return {path:launchGrantPath, sha256:grantSha, json:grant,
    invocation_manifest_path:invocationManifestPath, invocation_manifest_sha256:manifestSha, invocation_manifest:manifest};
}
const launchGrant = readLaunchGrant();
if (maybeLstat(runRoot)) throw new Error('attempt root already exists');
fs.mkdirSync(runRoot, { mode: 0o700 });
fs.chmodSync(runRoot, 0o700);
for (const p of [env.HOME, env.TMPDIR, actualRoot, actualOutputRoot]) { fs.mkdirSync(p, { mode: 0o700 }); fs.chmodSync(p, 0o700); }
let pre;
try {
  if (!maybeLstat(targetDir) || !fs.existsSync(cargoHome)) throw new Error('the reviewed R2 target and private package cache must exist');
  if (fs.readdirSync(actualRoot).length !== 1 || fs.readdirSync(actualOutputRoot).length !== 0 || maybeLstat(captureReceiptPath)) throw new Error('actual capture roots are not fresh and empty');
  pre = snapshot('before');
  if (pre.cache.holla_tag_capture_binary.path !== captureTestBinaryPath || pre.cache.holla_tag_capture_binary.sha256 !== captureTestBinarySha256) throw new Error('reused capture test binary identity changed');
  const command = { executable: cargo, argv: [cargo].concat(args), cwd: packageRoot, environment: env, timeout_ms: timeoutMs, raw_output_limit_bytes_total: maxRawOutputBytes, retry_count: 0, fail_fast: false, target: 'termrock-e2e integration test holla_tag_capture::captures_holla_from_pinned_immutable_tag (one ignored test)', frozen_suite_digest: expectedDigest, freeze_sha256: expectedFreezeSha, r13_runner_executed:false };
  fs.writeFileSync(path.join(runRoot, 'preflight.json'), JSON.stringify({ run_root: runRoot, created_at_utc: new Date(startedWall).toISOString(), launch_grant:launchGrant,
    invocation_manifest:{path:launchGrant.invocation_manifest_path,sha256:launchGrant.invocation_manifest_sha256},
    source_snapshot: pre, command: command, cache_sharing_note: 'CARGO_HOME, registry, and git are private real directories. The prefetch proof was made for the same unchanged Cargo.lock and the R2 root package; its checked lock closure is 206 registry archive/index entries and five Git checkouts. This capture command is strictly locked and offline.' }, null, 2) + '\n', { mode: 0o600, flag: 'wx' });
} catch (e) {
  fs.writeFileSync(path.join(runRoot, 'preflight-error.json'), JSON.stringify({ at_utc: new Date().toISOString(), error: String(e && e.stack || e) }, null, 2) + '\n', { mode: 0o600 });
  process.stderr.write(String(e && e.stack || e) + '\n');
  process.exit(125);
}
const stdoutPath = path.join(runRoot, 'stdout.log');
const stderrPath = path.join(runRoot, 'stderr.log');
const eventsPath = path.join(runRoot, 'process-events.jsonl');
const out = fs.openSync(stdoutPath, 'wx', 0o600);
const err = fs.openSync(stderrPath, 'wx', 0o600);
const events = fs.openSync(eventsPath, 'wx', 0o600);
function event(type, extra) { fs.writeSync(events, JSON.stringify(Object.assign({ at_utc: new Date().toISOString(), type: type }, extra || {})) + '\n'); }
const launchAt = Date.now();
event('spawn-request', { executable: cargo, argv: [cargo].concat(args), cwd: packageRoot, timeout_ms: timeoutMs, raw_output_limit_bytes_total: maxRawOutputBytes });
process.stdout.write('RUNNING ' + path.basename(runRoot) + ' started=' + new Date(launchAt).toISOString() + ' timeout=1200s output_limit_total=8388608\n');
const child = spawn(cargo, args, { cwd: packageRoot, env: env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
const childPid = child.pid;
event('spawned', { pid: childPid, process_group: childPid });
let timedOut = false;
let rawOutputLimitExceeded = false;
let rawOutputBytes = 0;
let terminationReason = null;
let closeResult = null;
let graceComplete = false;
let finished = false;
let timeoutTimer;
let killTimer;
let progressTimer;
function terminateGroup(reason) {
  if (terminationReason) return;
  terminationReason = reason;
  if (reason === 'timeout') timedOut = true;
  if (reason === 'raw-output-limit') rawOutputLimitExceeded = true;
  event(reason + '-sigterm', { elapsed_ms: Date.now() - launchAt });
  try { process.kill(-childPid, 'SIGTERM'); } catch (e) { event(reason + '-sigterm-error', { error: String(e) }); }
  killTimer = setTimeout(function() {
    event(reason + '-sigkill', { elapsed_ms: Date.now() - launchAt });
    try { process.kill(-childPid, 'SIGKILL'); } catch (e) { event(reason + '-sigkill-error', { error: String(e) }); }
    graceComplete = true;
    finish();
  }, 5000);
}
function captureRaw(fd, chunk, streamName) {
  const remaining = Math.max(0, maxRawOutputBytes - rawOutputBytes);
  const take = Math.min(chunk.length, remaining);
  if (take > 0) {
    const bytes = chunk.subarray(0, take);
    let offset = 0;
    while (offset < bytes.length) {
      const written = fs.writeSync(fd, bytes, offset, bytes.length - offset);
      if (written <= 0) throw new Error('raw log write made no progress');
      offset += written;
    }
    rawOutputBytes += offset;
  }
  if (take < chunk.length && !rawOutputLimitExceeded) {
    event('raw-output-limit-exceeded', { stream: streamName, total_captured_bytes: rawOutputBytes, limit_bytes: maxRawOutputBytes, dropped_chunk_bytes: chunk.length - take });
    terminateGroup('raw-output-limit');
  }
}
child.stdout.on('data', function(chunk) { captureRaw(out, chunk, 'stdout'); });
child.stderr.on('data', function(chunk) { captureRaw(err, chunk, 'stderr'); });
function finish() {
  if (finished || !closeResult || (terminationReason && !graceComplete)) return;
  finished = true;
  clearTimeout(timeoutTimer);
  clearTimeout(killTimer);
  clearInterval(progressTimer);
  fs.closeSync(out); fs.closeSync(err);
  const endAt = Date.now();
  let post = null; let postError = null;
  try { post = snapshot('after'); } catch (e) { postError = String(e && e.stack || e); }
  if (post) fs.writeFileSync(path.join(runRoot, 'postflight.json'), JSON.stringify(post, null, 2) + '\n', { mode: 0o600, flag: 'wx' });
  const stdoutSha = shaFile(stdoutPath); const stderrSha = shaFile(stderrPath);
  const stdoutText = fs.readFileSync(stdoutPath, 'utf8');
  const stderrText = fs.readFileSync(stderrPath, 'utf8');
  const nextestText = stdoutText + '\n' + stderrText;
  const nextestRunId = (nextestText.match(/Nextest run ID\s*:?\s*([0-9a-f-]+)/i) || [])[1] || null;
  const summaryLines = nextestText.split(/\r?\n/).filter(function(line) { return /Summary|tests run:|Starting [0-9]+ tests/.test(line); });
  const runCountMatch = nextestText.match(/\b(\d+)\s+tests?\s+run\b/i);
  const testsRun = runCountMatch ? Number(runCountMatch[1]) : null;
  const selectionCountOk = testsRun === expectedTestCount;
  const inputStable = Boolean(post && JSON.stringify(pre.package_files) === JSON.stringify(post.package_files)
    && pre.suite_digest === post.suite_digest && pre.freeze_sha256 === post.freeze_sha256
    && pre.runner.sha256 === post.runner.sha256 && JSON.stringify(pre.tools) === JSON.stringify(post.tools)
    && JSON.stringify(pre.sdk) === JSON.stringify(post.sdk) && JSON.stringify(pre.renderer) === JSON.stringify(post.renderer)
    && JSON.stringify(pre.reviews) === JSON.stringify(post.reviews)
    && JSON.stringify(pre.git) === JSON.stringify(post.git) && JSON.stringify(pre.tag) === JSON.stringify(post.tag)
    && JSON.stringify(pre.validator) === JSON.stringify(post.validator)
    && pre.cache.cache_verification_sha256 === post.cache.cache_verification_sha256
    && pre.cache.prefetch_completion_sha256 === post.cache.prefetch_completion_sha256
    && pre.cache.cargo_home_realpath === post.cache.cargo_home_realpath
    && JSON.stringify(pre.cache.registry) === JSON.stringify(post.cache.registry)
    && JSON.stringify(pre.cache.git) === JSON.stringify(post.cache.git)
    && pre.cache.target_dir_realpath === post.cache.target_dir_realpath
    && pre.cache.target_dir_device === post.cache.target_dir_device && pre.cache.target_dir_inode === post.cache.target_dir_inode
    && JSON.stringify(pre.cache.holla_tag_capture_binary) === JSON.stringify(post.cache.holla_tag_capture_binary));
  const captureGateOk = Boolean(post && post.capture.receipt_exists && post.capture.artifact_count === 40
    && post.capture.interaction_assertions === 12 && post.capture.capture_status === 'COMPLETE'
    && post.capture.execution_anchor_status === 'unverified' && post.capture.qualification_status === 'blocked'
    && post.capture.admission_status === 'NOT_RUN' && post.cache.holla_tag_capture_binaries.length === 1
    && post.cache.holla_tag_capture_binary.sha256 === captureTestBinarySha256);
  const wrapperExitCode = !inputStable || postError || rawOutputLimitExceeded
    || (closeResult.code === 0 && (!selectionCountOk || !captureGateOk)) ? 125
    : timedOut ? 124 : (closeResult.code === null ? 125 : closeResult.code);
  const receipt = {
    schema: 'termrock-rust-execution-receipt-v1',
    run_root: runRoot,
    runner: pre.runner,
    command: { executable: cargo, argv: [cargo].concat(args), cwd: packageRoot, environment: env },
    process: { pid: childPid, process_group: childPid, started_at_utc: new Date(launchAt).toISOString(), ended_at_utc: new Date(endAt).toISOString(), elapsed_ms: endAt - launchAt, timeout_ms: timeoutMs, timed_out: timedOut, raw_output_limit_exceeded: rawOutputLimitExceeded, termination_reason: terminationReason, raw_output_limit_bytes_total: maxRawOutputBytes, raw_output_captured_bytes_total: rawOutputBytes, exit_code: closeResult.code, signal: closeResult.signal, wrapper_exit_code: wrapperExitCode },
    nextest_run_id: nextestRunId,
    launch_grant: {path:launchGrant.path,sha256:launchGrant.sha256},
    invocation_manifest: {path:launchGrant.invocation_manifest_path,sha256:launchGrant.invocation_manifest_sha256},
    summary_lines: summaryLines,
    result_counts: { expected_tests: expectedTestCount, observed_tests: testsRun, selection_count_ok: selectionCountOk, summary_lines: summaryLines, capture_gate_ok:captureGateOk },
    input_stable: inputStable,
    before_snapshot: 'preflight.json',
    after_snapshot: post ? 'postflight.json' : null,
    postflight_error: postError,
    raw_outputs: { stdout: { path: stdoutPath, sha256: stdoutSha, bytes: fs.statSync(stdoutPath).size }, stderr: { path: stderrPath, sha256: stderrSha, bytes: fs.statSync(stderrPath).size }, process_events: eventsPath, captured_bytes_total: rawOutputBytes, limit_bytes_total: maxRawOutputBytes },
    cache_sharing_note: 'CARGO_HOME, registry, and git are private real directories. The prefetch proof was made for the same unchanged Cargo.lock and R2 root package; the capture command is strictly locked and offline.',
    r13_runner_executed:false,
    evidence_limit: 'This run only tests actual output capture from an already built immutable-tag executable. It does not build or requalify that product, does not accept expected data, does not perform admission, and leaves qualification blocked and the execution anchor unverified.',
    capture: post ? post.capture : null
  };
  fs.closeSync(events);
  fs.writeFileSync(path.join(runRoot, 'final-receipt.json'), JSON.stringify(receipt, null, 2) + '\n', { mode: 0o600, flag: 'wx' });
  process.stdout.write(JSON.stringify({ run_root: runRoot, nextest_run_id: nextestRunId, elapsed_ms: endAt - launchAt, exit_code: closeResult.code, signal: closeResult.signal, timed_out: timedOut, raw_output_limit_exceeded: rawOutputLimitExceeded, raw_output_captured_bytes_total: rawOutputBytes, input_stable: inputStable, expected_tests: expectedTestCount, observed_tests: testsRun, selection_count_ok: selectionCountOk, summary_lines: summaryLines, stdout_sha256: stdoutSha, stderr_sha256: stderrSha, wrapper_exit_code: wrapperExitCode, receipt: path.join(runRoot, 'final-receipt.json') }, null, 2) + '\n');
  process.exitCode = wrapperExitCode;
}
child.on('error', function(e) { event('child-error', { error: String(e) }); closeResult = { code: null, signal: null }; finish(); });
child.on('close', function(code, signal) { closeResult = { code: code, signal: signal }; event('child-close', { exit_code: code, signal: signal, timed_out: timedOut }); finish(); });
timeoutTimer = setTimeout(function() { terminateGroup('timeout'); }, timeoutMs);
progressTimer = setInterval(function() { process.stdout.write('STILL_RUNNING elapsed_seconds=' + Math.floor((Date.now() - launchAt) / 1000) + '\n'); }, 20000);
