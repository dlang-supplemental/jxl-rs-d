# Machine / environment memories

| Fact | Uses |
|------|------|
| Pattern for Rust→D on dub: `vello-d` style bridge crate (`cdylib`+`staticlib`) + `preBuildCommands` cargo release build + D `extern(C)` | 1 |
| Official browser JXL decode path is `jxl-rs` (`jxl` crate on crates.io), not libjxl C++ | 1 |
| `jxl` crate is decode-only (no encode API) | 1 |
