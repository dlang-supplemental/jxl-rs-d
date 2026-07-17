module jxl_rs.bindings;

@nogc nothrow:

extern (C):

/// Status codes from the Rust bridge.
enum JxlRsStatus
{
    ok = 0,
    needMoreInput = 1,
    error = 2,
}

/// Basic image header info from `jxl_rs_probe`.
struct JxlRsImageInfo
{
    uint width;
    uint height;
    ubyte hasAlpha;
    ubyte isAnimated;
    ubyte valid;
}

/// One-shot decode to tightly packed RGBA8. Free with `jxl_rs_free(ptr, width*height*4)`.
ubyte* jxl_rs_decode_rgba8(const(ubyte)* data, size_t len, uint* outWidth, uint* outHeight);

/// Free a buffer returned by `jxl_rs_decode_rgba8`.
void jxl_rs_free(ubyte* ptr, size_t len);

/// Thread-local last error message (NUL-terminated). Valid until the next bridge call.
const(char)* jxl_rs_last_error();

/// Probe headers without allocating pixels.
JxlRsStatus jxl_rs_probe(const(ubyte)* data, size_t len, JxlRsImageInfo* outInfo);
