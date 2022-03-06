struct A {
    u: u32;
    v: u32;
    w: vec2<u32>;
    [[size(16), align(8)]]
    x: u32;
    xx: u32;
};

struct B {
    a: vec2<u32>;
    b: vec3<u32>;
    c: u32;
    d: u32;
    [[align(16)]]
    e: A;
    f: vec3<u32>;
    g: array<A, 3>;
    h: i32;
    [[align(32)]]
    i: array<A>;
};

[[group(0), binding(0)]]
var<storage> in: B;

[[group(0), binding(1)]]
var<storage, read_write> out: B;

[[stage(compute), workgroup_size(1, 1, 1)]]
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