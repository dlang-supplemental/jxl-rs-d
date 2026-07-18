# jxl-rs-d

D bindings for [jxl-rs](https://github.com/libjxl/jxl-rs), the memory-safe
Rust JPEG XL *decoder* used by Chromium and Firefox.

Encode is intentionally out of scope. For encode + a unified D API, see
[`jxl-d`](https://github.com/dlang-supplemental/jxl-d) (libjxl encode + this
package for decode).

## Site

https://dlang-supplemental.github.io/jxl-rs-d/

## Requirements

* D compiler (`ldc2` or `dmd`) and [dub](https://dub.pm/)
* [Rust toolchain](https://rustup.rs/) + Cargo (edition 2024 capable; rustc 1.85+)
* Network on first `cargo build` to fetch crates.io deps

## Quick start

```d
import jxl_rs;

void main()
{
    auto img = decodeRgba8(cast(immutable(ubyte)[]) std.file.read("photo.jxl"));
    // img.width, img.height, img.pixels (RGBA8, tightly packed)
}
```

Build / test:

```bash
dub test
```

The `preBuildCommands` in `dub.sdl` compile `jxl_rs_bridge` (Rust `cdylib` /
`staticlib`) before the D sources.

## Layout

* `jxl_rs_bridge/` — C ABI wrapper around the `jxl` crate
* `source/jxl_rs/` — D `extern(C)` bindings + high-level helpers

## Changelog

See [CHANGELOG](CHANGELOG.adoc).

## License

BSD-3-Clause (matches jxl-rs).
