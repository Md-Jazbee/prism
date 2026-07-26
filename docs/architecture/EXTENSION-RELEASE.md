# Extension release workflow (P8)

## Versioning

- Extension `version` tracks engine major in the leftmost component when possible (`0.8.x` during P8).
- Handshake: `prism.engineMajor` must equal `/health.api_version` major.

## Build artifacts

```bash
pnpm install
pnpm --filter @prism/graph-view build
pnpm --filter prism-vscode build
pnpm --filter prism-vscode test
pnpm --filter prism-vscode package   # → prism-vscode-*.vsix
```

## CI

`.github/workflows/extension.yml`: lint (typecheck), unit tests, build, `vsce package` artifact.

## Signing / verification

- VSIX published via Marketplace / Open VSX publisher account (manual for now).
- Binary download manifest: `extensions/vscode/binaries/manifest.json` + checksums when release binaries exist.

## Changelog

Keep extension-facing notes in `extensions/vscode/CHANGELOG.md`.
