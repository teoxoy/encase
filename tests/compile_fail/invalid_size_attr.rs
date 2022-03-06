use encase::WgslType;

fn main() {}

#[derive(WgslType)]
struct Test {
    #[size]
    a: u32,
    #[size()]
    b: u32,
    #[size(invalid)]
    c: u32,
    #[size(-1)]
    d: u32,
}
