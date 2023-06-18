@group(0) @binding(0)
var<storage> in: B;

@group(0) @binding(1)
var<storage, read_write> out: B;

@compute @workgroup_size(1, 1, 1)
fn main() {
    out.a = in.a;
    out.b = in.b;
    out.c = in.c;
    out.d = in.d;
    out.e = in.e;
    out.f = in.f;
    out.g[0] = in.g[0];
    out.g[1] = in.g[1];
    out.g[2] = in.g[2];
    out.h = in.h;
    for (var i = 0u; i < arrayLength(&in.i); i = i + 1u) {
        out.i[i] = in.i[i];
    }
}