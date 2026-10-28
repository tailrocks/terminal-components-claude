#!/usr/bin/env python3
import datetime, hashlib, json, os, pathlib, shutil, stat, sys, tempfile, time
INV=pathlib.Path('/private/tmp/pr38-consolidation-inventory-20261029')
SRC=INV/'downloads-common-odb-closure-v2'
MAP=SRC/'downloads-new-vs-canonical.tsv'
LOCAL=SRC/'downloads-local-loose-objects.tsv'
SEAL=SRC/'REPORT-SEAL.json'
DECISION=INV/'sol-high-downloads-odb-screen-v1/decision.json'
REPO=pathlib.Path('/Users/donbeave/Projects/termrock-pr38-shared-20261028')
BASE=REPO/'docs/consolidation/2026-10-29/git/tcc'
OUT=BASE/'odb-groups/downloads-terminal-components-claude'
CHUNK=48*1024*1024

def sha(path):
 h=hashlib.sha256(); n=0
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''): h.update(b);n+=len(b)
 return h.hexdigest(),n

def write_json(path, value):
 path.parent.mkdir(parents=True,exist_ok=True)
 path.write_text(json.dumps(value,sort_keys=True,indent=2)+'\n')

def copy_evidence(src, dest, expected):
 h,n=sha(src)
 if h!=expected: raise RuntimeError(f'evidence source digest mismatch: {src} {h} != {expected}')
 dest.parent.mkdir(parents=True,exist_ok=True)
 shutil.copyfile(src,dest)
 os.chmod(dest,0o644)
 h2,n2=sha(dest)
 if h2!=h or n2!=n: raise RuntimeError(f'evidence copy mismatch: {dest}')
 return {'path':dest.relative_to(OUT).as_posix(),'sha256':h,'size_bytes':n}

if OUT.exists(): raise SystemExit(f'refusing overwrite existing archive group: {OUT}')
start=datetime.datetime.now(datetime.timezone.utc)
REPO_HEAD=os.popen(f'git -C {REPO} rev-parse HEAD').read().strip()
if REPO_HEAD!='23ad1dbba498ede584d6eb2d7b4203f19ae1bb8f': raise SystemExit(f'unexpected canonical preimage {REPO_HEAD}')
# Parse approved source map and independent Safety decision.
rows=[]
for line in MAP.read_text().splitlines():
 if not line: continue
 cols=line.split('\t')
 if len(cols)!=6: raise RuntimeError(f'invalid source map row: {line[:120]}')
 oid,typ,logical,rawsha,rawbytes,source_rel=cols
 rows.append({'oid':oid,'type':typ,'logical_bytes':int(logical),'compressed_sha256':rawsha,'compressed_bytes':int(rawbytes),'source_relative_path':source_rel})
dec=json.loads(DECISION.read_text())
if dec.get('decision')!='ELIGIBLE_PROJECT_OBJECT_TRANSPORT': raise RuntimeError('Safety decision is not eligibility')
by_oid={x['oid']:x for x in dec['objects']}
if len(rows)!=3747 or len(by_oid)!=3747: raise RuntimeError('unexpected new-object count')
# Confirm canonical typed base and full local loose-object inventory.
canonical_path=BASE/'inputs/physical-objects.tsv'
canonical_sha,canonical_bytes=sha(canonical_path)
if canonical_sha!='1e1094407122bfe6814f028cc360101490093ae7a3c4d5af589c808ff887beab': raise RuntimeError('canonical typed-object base changed')
canonical={}
for line in canonical_path.read_text().splitlines():
 oid,typ,sz=line.split('\t'); canonical[oid]=(typ,int(sz))
local_rows=[]
for line in LOCAL.read_text().splitlines():
 oid,typ,sz,rawsha,rawbytes=line.split('\t'); local_rows.append((oid,typ,int(sz),rawsha,int(rawbytes)))
if len(local_rows)!=5012 or len({x[0] for x in local_rows})!=5012: raise RuntimeError('local typed object inventory mismatch')
local={x[0]:(x[1],x[2]) for x in local_rows}
if len(set(local)-set(canonical))!=3747 or len(set(local)&set(canonical))!=1265: raise RuntimeError('canonical/local OID set arithmetic mismatch')
for oid,(typ,size) in local.items():
 if oid in canonical and canonical[oid]!=(typ,size): raise RuntimeError(f'canonical type/size conflict: {oid}')
# Check new map is exactly the local set difference and agrees with Safety's reviewed object identities.
if {x['oid'] for x in rows} != set(local)-set(canonical): raise RuntimeError('new-object map does not equal canonical set difference')
for r in rows:
 if local[r['oid']]!=(r['type'],r['logical_bytes']): raise RuntimeError(f'new object type/size mismatch: {r["oid"]}')
 q=by_oid[r['oid']]
 if (q['type'],q['logical_bytes'],q['compressed_sha256'],q['compressed_bytes'])!=(r['type'],r['logical_bytes'],r['compressed_sha256'],r['compressed_bytes']):
  raise RuntimeError(f'Safety decision/source map mismatch: {r["oid"]}')
# Copy approved source evidence unchanged.
OUT.mkdir(parents=True)
evidence=OUT/'evidence/source-v2'
expected_files={
 'README.md':'8c45e0071f8e39830f8ece57f572f5ba3c3f7e8a83693f16663e7bccb4ce4499',
 'downloads-local-loose-objects.tsv':'38b46d8008c80f1cfaa82078fec913e1da132757540d4cecf99eeed0bdf038fd',
 'downloads-loose-file-errors.jsonl':'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
 'downloads-new-vs-canonical.tsv':'9340f7bb886247105a78dddf7ef8c28deddc585bbf740394d7255ccbe7063bbf',
 'downloads-object-conflicts.jsonl':'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
 'downloads-odb-closure.json':'37ae2286c5376ecd792455728d290062b4f328a3fd68c8511cecbff1c2ce3b3e',
 'source-alternates.raw':'50dea5eee6a94e736f758b01c7aafc013a67cf57e67a2196f67cc9d9bf9673c8',
 'REPORT-SEAL.json':'5880b5b81dcf7b5cdd8130b2c458a475e74b062210b9dcd5e104c8de1346abc8',
 'SHA256SUMS':'ae8ce3c2d7512c780e3cc8de2b078ec110fc948f338cf6480db4cfe7a15903aa'}
for name,expected in expected_files.items(): copy_evidence(SRC/name,evidence/name,expected)
screen_dest=OUT/'evidence/safety/downloads-odb-screen-v1-decision.json'
screen_sha,_=sha(DECISION)
if screen_sha!='62741c0691cbe965afc0621738e2ed796ecf3e8e9fc3fa65a04a9aa8e935030b': raise RuntimeError('unexpected Safety decision hash')
copy_evidence(DECISION,screen_dest,screen_sha)
# Preserve the full 3747 typed object delta as exact raw compressed loose bytes, chunking any >48 MiB.
objdir=OUT/'objects/loose'; chunkdir=OUT/'objects/chunks'
chunk_rows=[]; map_rows=[]; copied=0; chunked=[]
for r in sorted(rows,key=lambda x:x['oid']):
 oid=r['oid']; rel=r['source_relative_path']
 src=SRC/rel
 before=os.lstat(src)
 if not stat.S_ISREG(before.st_mode) or before.st_size!=r['compressed_bytes']: raise RuntimeError(f'raw loose source type/size mismatch: {rel}')
 h=hashlib.sha256(); n=0
 if r['compressed_bytes']<=CHUNK:
  dest=objdir/oid[:2]/oid[2:]; dest.parent.mkdir(parents=True,exist_ok=True)
  with src.open('rb') as inp, dest.open('wb') as out:
   for b in iter(lambda:inp.read(1024*1024),b''): h.update(b);n+=len(b);out.write(b)
  os.chmod(dest,0o644)
  storage='loose-path'; storage_path=dest.relative_to(OUT).as_posix(); chunk_count=0
 else:
  storage='ordered-chunks'; storage_path='objects/chunks/index.tsv'; chunk_count=0
  with src.open('rb') as inp:
   seq=0
   while True:
    b=inp.read(CHUNK)
    if not b: break
    seq+=1; h.update(b);n+=len(b)
    ch=hashlib.sha256(b).hexdigest()
    dest=chunkdir/f'{oid}-{seq:06d}-{ch}.chunk'; dest.parent.mkdir(parents=True,exist_ok=True)
    dest.write_bytes(b);os.chmod(dest,0o644)
    if dest.stat().st_size>CHUNK: raise RuntimeError(f'chunk over limit: {dest}')
    if sha(dest)!=(ch,len(b)): raise RuntimeError(f'chunk hash mismatch: {dest}')
    chunk_rows.append((oid,seq,dest.relative_to(OUT).as_posix(),len(b),ch))
  chunk_count=seq; chunked.append({'oid':oid,'chunk_count':seq})
 after=os.lstat(src)
 a=(before.st_dev,before.st_ino,before.st_mode,before.st_size,before.st_mtime_ns,before.st_ctime_ns)
 z=(after.st_dev,after.st_ino,after.st_mode,after.st_size,after.st_mtime_ns,after.st_ctime_ns)
 if a!=z: raise RuntimeError(f'raw loose source changed during read: {rel}')
 if h.hexdigest()!=r['compressed_sha256'] or n!=r['compressed_bytes']: raise RuntimeError(f'raw loose hash mismatch: {oid}')
 if storage=='loose-path':
  dest=OUT/storage_path
  if sha(dest)!=(h.hexdigest(),n): raise RuntimeError(f'copied loose object mismatch: {oid}')
 map_rows.append((oid,r['type'],r['logical_bytes'],r['compressed_sha256'],r['compressed_bytes'],storage,storage_path,chunk_count))
 copied+=1
# Write portable reconstruction map.
with (OUT/'portable-object-map.tsv').open('w') as f:
 f.write('oid\ttype\tlogical_bytes\tcompressed_sha256\tcompressed_bytes\tstorage_kind\tstorage_path\tchunk_count\n')
 for x in map_rows: f.write('\t'.join(map(str,x))+'\n')
with (chunkdir/'index.tsv').open('w') as f:
 f.write('oid\tsequence\trelative_chunk_path\tbytes\tsha256\n')
 for x in chunk_rows: f.write('\t'.join(map(str,x))+'\n')
# Independently reassemble the six large objects from staged chunks and verify exact raw hashes.
for item in chunked:
 oid=item['oid']; desthash=next(x['compressed_sha256'] for x in rows if x['oid']==oid); destsize=next(x['compressed_bytes'] for x in rows if x['oid']==oid)
 h=hashlib.sha256(); n=0
 for _,seq,path,sz,ch in [x for x in chunk_rows if x[0]==oid]:
  p=OUT/path; d=p.read_bytes()
  if hashlib.sha256(d).hexdigest()!=ch or len(d)!=sz: raise RuntimeError(f'chunk recon input mismatch: {path}')
  h.update(d);n+=len(d)
 if h.hexdigest()!=desthash or n!=destsize: raise RuntimeError(f'chunk reconstruction mismatch: {oid}')
# Create scoped manifest and documentation.
manifest={
 'schema':'pr38-downloads-tcc-new-odb-delta-v1',
 'status':'partial object-delta custody; worktree/ref/admin state and remaining untracked/ignored files are separate pending groups',
 'source_identity':{'repository_root':'/Users/donbeave/Downloads/terminal-components-claude','common_dir':'/Users/donbeave/Downloads/terminal-components-claude/.git','head':'b9a6507e9894d32f0a2c10a819981e811e554b8e','branch':'parity/termrock-visual-baseline','object_format':'sha1'},
 'canonical_dependency':{'archive_relative_root':'../..','typed_oid_manifest_relative_path':'../../inputs/physical-objects.tsv','typed_oid_manifest_sha256':canonical_sha,'typed_oid_manifest_git_blob':'4d17cbc4dceac37b9b10ff01ec24ca5713a1b91d','typed_oid_count':len(canonical),'canonical_pack_external_reconstruction_review':'/private/tmp/pr38-tcc-remote-reconstruction-20261029-v1/decision.json','canonical_pack_review_sha256':'937657498f06cd140e9757604965f1a7684d11d00202b4e43b20c50505553e88'},
 'local_object_set':{'local_loose_oid_count':len(local),'local_loose_inventory_sha256':'38b46d8008c80f1cfaa82078fec913e1da132757540d4cecf99eeed0bdf038fd','canonical_overlap_count':1265,'new_oid_count':len(rows),'new_oid_map_source_sha256':'9340f7bb886247105a78dddf7ef8c28deddc585bbf740394d7255ccbe7063bbf','new_oids_all_type_size_checked_against_canonical':True,'type_size_conflicts':0,'missing_graph_links_against_canonical_plus_local':0},
 'payload':{'raw_new_loose_objects_preserved':copied,'compressed_bytes':sum(x['compressed_bytes'] for x in rows),'logical_object_bytes':sum(x['logical_bytes'] for x in rows),'large_object_chunk_threshold_bytes':CHUNK,'chunked_oids':chunked,'chunk_count':len(chunk_rows),'chunk_reassembly_hashes_verified':True,'portable_map':'portable-object-map.tsv','chunk_map':'objects/chunks/index.tsv'},
 'eligibility':{'decision':'ELIGIBLE_PROJECT_OBJECT_TRANSPORT','review_path':'evidence/safety/downloads-odb-screen-v1-decision.json','review_sha256':screen_sha,'review_scope':'3747 new ODB objects only; not full repository/worktree/all-work acceptance'},
 'source_evidence':{name:{'relative_path':f'evidence/source-v2/{name}','sha256':h} for name,h in expected_files.items()},
 'restoration':{'procedure':'Start with this repository archive’s canonical TCC object store at archive_relative_root; copy small objects to Git objects fanout path under objects/loose/, and reassemble chunked objects by ordered sequence from objects/chunks/index.tsv before installing them at the same OID fanout path.','raw_loose_encoding':'All preserved bytes are the exact source compressed loose-object bytes; no decoding or recompression occurred in the archived payload.'},
 'limits':['This is not a standalone clone; it requires the canonical TCC archive payload at the relative dependency.','The 1265 OIDs already in the canonical typed set are not duplicated here; their semantic object bytes are supplied by the canonical group.','This group does not yet contain Downloads refs, reflogs, indexes, sparse patterns, worktree source files, or untracked/ignored content.','The 21 newly identified untracked/ignored payload groups are pending separate privacy review/publication.','No local source Git store was modified.','No cleanup/deletion or full-custody completion is claimed.']}
write_json(OUT/'manifest.json',manifest)
readme='''# Downloads TCC new-object delta (partial custody)\n\nThis group preserves the 3,747 loose Git objects in `/Users/donbeave/Downloads/terminal-components-claude/.git/objects` whose typed OIDs are absent from the canonical TCC object manifest. All bytes are the exact compressed source loose-object bytes; they are not decoded/recompressed.\n\nThe group depends on the canonical TCC archive at `../..` and its `inputs/physical-objects.tsv` (257,216 typed OIDs, SHA-256 `1e1094407122bfe6814f028cc360101490093ae7a3c4d5af589c808ff887beab`). The full Downloads loose inventory has 5,012 objects: 1,265 have matching canonical type/size and are represented by that dependency; the remaining 3,747 are stored here. The Safety review found no type/size conflicts or missing graph links for the canonical-plus-local object union.\n\nObjects at or below 48 MiB are stored at their Git fanout paths under `objects/loose/`. The three larger raw encodings are split into chunks no larger than 48 MiB; `portable-object-map.tsv` and `objects/chunks/index.tsv` bind ordered chunk paths and hashes so the exact raw source bytes can be reassembled. The staged group reassembled and hash-checked all three.\n\nThis commit contains only the ODB object delta. Refs, reflogs, 37 linked-worktree indexes/admin state, sparse-checkout patterns, branch/upstream metadata, main worktree files, and the remaining 21 untracked/ignored content groups are separate pending groups. It is not an all-work custody or cleanup-complete claim.\n'''
(OUT/'README.md').write_text(readme)
# Hash every file under group except the top checksum file.
files=sorted(p for p in OUT.rglob('*') if p.is_file() and p!=OUT/'SHA256SUMS')
with (OUT/'SHA256SUMS').open('w') as f:
 for p in files:
  hh,nn=sha(p);f.write(f'{hh}  {p.relative_to(OUT).as_posix()}\n')
# Verify exact file set and hashes.
verify=[]
for line in (OUT/'SHA256SUMS').read_text().splitlines():
 hh,rel=line.split('  ',1);p=OUT/rel
 actual,size=sha(p)
 if actual!=hh: raise RuntimeError(f'archive checksum mismatch: {rel}')
 verify.append(rel)
if set(verify)!={p.relative_to(OUT).as_posix() for p in files}: raise RuntimeError('archive file list mismatch')
end=datetime.datetime.now(datetime.timezone.utc)
receipt={
 'schema':'pr38-downloads-odb-delta-assembly-receipt-v1','started_utc':start.isoformat(),'ended_utc':end.isoformat(),
 'canonical_preimage':REPO_HEAD,'group_path_relative_to_repo':str(OUT.relative_to(REPO)),
 'source_map_sha256':hashlib.sha256(MAP.read_bytes()).hexdigest(),'source_report_seal_sha256':hashlib.sha256(SEAL.read_bytes()).hexdigest(),
 'safety_decision_sha256':screen_sha,'objects_copied':copied,'source_compressed_bytes':sum(x['compressed_bytes'] for x in rows),
 'chunks_written':len(chunk_rows),'chunk_size_max':max((x[3] for x in chunk_rows),default=0),
 'payload_files_before_receipt':len(files),'bytes_before_receipt':sum(p.stat().st_size for p in OUT.rglob('*') if p.is_file()),
 'portable_object_map_sha256':hashlib.sha256((OUT/'portable-object-map.tsv').read_bytes()).hexdigest(),
 'chunk_map_sha256':hashlib.sha256((chunkdir/'index.tsv').read_bytes()).hexdigest(),
 'all_pre_receipt_verifications_passed':True}
write_json(OUT/'assembly-receipt.json',receipt)
# Add receipt to SHA256SUMS and reseal, then verify. This makes receipt itself part of the committed file set.
files=sorted(p for p in OUT.rglob('*') if p.is_file() and p!=OUT/'SHA256SUMS')
with (OUT/'SHA256SUMS').open('w') as f:
 for p in files:
  hh,nn=sha(p);f.write(f'{hh}  {p.relative_to(OUT).as_posix()}\n')
for line in (OUT/'SHA256SUMS').read_text().splitlines():
 hh,rel=line.split('  ',1)
 if sha(OUT/rel)[0]!=hh: raise RuntimeError(f'final checksum mismatch: {rel}')
print(json.dumps({'group':str(OUT),'objects':copied,'object_bytes':sum(x['compressed_bytes'] for x in rows),'chunk_count':len(chunk_rows),'max_chunk_bytes':max((x[3] for x in chunk_rows),default=0),'group_file_count':len(files)+1,'group_bytes':sum(p.stat().st_size for p in OUT.rglob('*') if p.is_file()),'source_map_sha256':receipt['source_map_sha256'],'safety_decision_sha256':screen_sha,'portable_map_sha256':receipt['portable_object_map_sha256'],'checksum_sha256':hashlib.sha256((OUT/'SHA256SUMS').read_bytes()).hexdigest(),'receipt_sha256':hashlib.sha256((OUT/'assembly-receipt.json').read_bytes()).hexdigest(),'started_utc':receipt['started_utc'],'ended_utc':receipt['ended_utc']},sort_keys=True,indent=2))
