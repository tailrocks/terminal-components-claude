# T37 source history portability

This record checks the candidate history from a fresh local clone.

The clone has the requested branch, commit, and tree. It has no tags, alternates, or worker refs.

All eight original tips and raw foundation commit `2fee2c2` are ancestors of the candidate head. The first parent and candidate head have the same tree.

The read-only producer build validates 1,122 records and stable IDs. Its rows and coverage match the committed output. The only difference is the current `composition_input_commit` field.

The clone remains at `/Users/donbeave/Downloads/termrock-pr38-history-clone-20261026`. Its sparse checkout contains `docs` and `crates`.

No Cargo command, test command, or behavior case ran. Product behavior remains `NOT VERIFIED`. Independent review is pending.

See [verification.json](verification.json) for commit ancestry, blob hashes, commands, disk probes, and limits.
