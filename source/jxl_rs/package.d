module jxl_rs;

public import jxl_rs.bindings;

import std.exception : enforce;
import std.string : fromStringz;

/// Decoded tightly packed RGBA8 image.
struct Rgba8Image
{
    uint width;
    uint height;
    ubyte[] pixels;

    @property size_t byteLength() const pure @safe
    {
        return cast(size_t) width * height * 4;
    }
}

/// Exception thrown when the Rust bridge reports a decode error.
class JxlRsException : Exception
{
    this(string msg, string file = __FILE__, size_t line = __LINE__)
    {
        super(msg, file, line);
    }
}

private string lastErrorString()
{
    auto p = jxl_rs_last_error();
    if (p is null)
        return "unknown jxl-rs error";
    return fromStringz(p).idup;
}

/// Decode a complete JXL buffer to RGBA8.
Rgba8Image decodeRgba8(const(ubyte)[] data)
{
    uint w, h;
    auto ptr = jxl_rs_decode_rgba8(data.ptr, data.length, &w, &h);
    enforce!JxlRsException(ptr !is null, lastErrorString());
    const len = cast(size_t) w * h * 4;
    auto pixels = ptr[0 .. len].dup;
    jxl_rs_free(ptr, len);
    return Rgba8Image(w, h, pixels);
}

/// Probe width/height/alpha/animation without decoding pixels.
JxlRsImageInfo probe(const(ubyte)[] data)
{
    JxlRsImageInfo info;
    auto st = jxl_rs_probe(data.ptr, data.length, &info);
    enforce!JxlRsException(st == JxlRsStatus.ok && info.valid, lastErrorString());
    return info;
}

unittest
{
    // Invalid / empty input should fail cleanly.
    import std.exception : assertThrown;

    assertThrown!JxlRsException(decodeRgba8(null));
    assertThrown!JxlRsException(probe([0, 1, 2, 3]));
}
