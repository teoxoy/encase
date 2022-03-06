struct A {
    array_length: u32;
    array_length_call_ret_val: u32;
    a: vec3<u32>;
    [[align(16)]]
    arr: array<u32>;
};

[[group(0), binding(0)]]
var<storage> in: A;

[[group(0), binding(1)]]
var<storage, read_write> out: A;

[[stage(compute), workgroup_size(1, 1, 1)]]
fn main() {
    out.array_length = in.array_length;
    out.a = in.a;
    for (var i = 0u; i < arrayLength(&in.arr); i = i + 1u) {
        out.arr[i] = in.arr[i];
    }
    out.array_length_call_ret_val = arrayLength(&in.arr);
}