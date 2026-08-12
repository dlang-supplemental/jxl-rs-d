# Agent notes — jxl-rs-d

Project facts for agents. Workstation/env facts live only in `$CODE_ROOT/MEMORIES.md`.

- Rust→D on dub follows the `vello-d` pattern: bridge crate (`cdylib` + `staticlib`) + `preBuildCommands` cargo release build + D `extern(C)`
- Official browser JXL decode path is `jxl-rs` (`jxl` crate on crates.io), not libjxl C++
- `jxl` crate is decode-only (no encode API)
