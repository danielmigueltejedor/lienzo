# Lienzo Engine

`photocraft/` is the Rust engine, imported from `storytold/photocraft` at `909efc0f6df0a684d270019384e355c850642e42`. Licenses and notices in that tree stay. The desktop binary built there is named `lienzo`.

```bash
cargo run -p photocraft --manifest-path photocraft/Cargo.toml
```

`lienzo-gnome` still links the C++ engine. This directory does not delete that path.

```bash
cargo test -p lienzo-engine
```

The suite contract is [docs/creative-suite-architecture.md](../docs/creative-suite-architecture.md).
