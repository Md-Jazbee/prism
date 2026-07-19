# Fingerprint algorithm (P0 Stage A)

## File hash

- Algorithm: **XXH3-128** (`xxhash-rust`)
- Input: full file bytes
- Encoding: lowercase hex of big-endian u128
- Empty file: still hashed (defined digest)

## Directory / tree Merkle

1. Discover files (gitignore + ignore policy).
2. For each file: `(repo_relative_path, content_hash)`.
3. Sort by path.
4. Concatenate `path \\0 hash \\n` for all leaves; XXH3-128 → `tree_fingerprint`.

**Unchanged subtree skip (contract):** if a file's stored `content_hash` equals the fresh hash, skip parse-hook and subgraph replace. Directory-level skip can be added later by caching per-directory Merkle nodes; P0 uses per-file skip.

## Snapshot identity

| Field | Meaning |
|---|---|
| `git_commit` | `git rev-parse HEAD` or null |
| `dirty` | non-empty `git status --porcelain` |
| `tree_fingerprint` | Merkle root above |

Clean commit vs dirty worktree are **distinguishable** even when the tree hash matches a prior clean snapshot.
