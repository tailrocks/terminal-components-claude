# Termrock history preservation

This commit is an archival ref, not a product integration or release. Its first parent is the public `main` snapshot `032e9faa3ab12e98fef3b3835757eba5f1feee43`. The tree adds only this note; five additional parents keep the exact Termrock source histories reachable. Do not merge this branch into `main`.

## Preserved parent tips

- `e44d5504e91d90f8404a5488d8539ed71001e31f` — PR17 visibility handoff fix; this tip had no ancestry in the live remote refs at review time.
- `4b473a98a8641a9dae8dbf9c31c0c94b15465496` — measured visibility status and evidence; includes the vis02 test heads `3c2ffed36063b068a03df93b1cd0c8d4e7c8b59f` and `b682cb26d68b353aeeccf9e51653eddf097b39f5` as ancestors.
- `bac19aad80bcab6e439d88216eedabbf1c3efe35` — bounded Rust command runner; includes `331b6fd95835ca505a9417581aaa37df52504797` as an ancestor. This tip had no ancestry in the live remote refs at review time.
- `81a8bf15cd3042f80649e2b48fed479829518dbd` — mise build setup.
- `fb1295f3074701e40b96314478651bf277a99ed6` — nested Nextest child outcomes; it also retains the external implementation `0327b42f11565ddd47121d9279d879396ea9d846` and research note `38a897b179b6f01cf07dd6d889f7a278886e5226` as ancestors.

The annotated `visual-baseline` tag `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` already exists on the live remote and peels to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.

## Audit scope

On 2026-10-16, all 62 advertised refs were checked, including pull-request and recovery refs. The 157 objects missing from the local root-store view and 154 missing from the vis02 view were recovered from remote objects; the selected histories then passed connectivity checks. The 19 product blobs not already reachable from live refs were scanned for common credential and key patterns; no matches were found.

This ref preserves Git commit ancestry only. It does not certify that temporary object stores contain no other useful refs or non-Git/LFS/worktree data, and it is not approval to delete those stores.
