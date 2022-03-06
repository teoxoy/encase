use encase::WgslType;

fn main() {}

#[derive(WgslType)]
struct Test {
    #[align]
    a: u32,
    #[align()]
    b: u32,
    #[align(invalid)]
    c: u32,
    #[align(3)]
    d: u32,
}
