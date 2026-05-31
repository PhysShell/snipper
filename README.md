# Snipper

Portable structural expansion engine. Resolves a trigger at the cursor into
expanded candidates by parsing the CST context, matching templates, and
emitting `TextEdit` patches.

## Status

Early scaffold. Engine implementation follows the main Snipper spec.

## Quick start

```text
cargo build --workspace
just verify
```

## Architecture

See [`docs/architecture.md`](docs/architecture.md).

## Agent guide

See [`AGENTS.md`](AGENTS.md).

## License

MIT — see [`LICENSE`](LICENSE).
