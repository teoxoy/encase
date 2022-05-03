#![feature(trivial_bounds)]

use core::cell::Cell;
use encase::ShaderType;
use std::{borrow::Cow, rc::Rc, sync::Arc};

fn main() {}

#[derive(ShaderType)]
struct Test<'a> {
    a: &'a u32,
    b: &'a mut u32,
    c: Box<u32>,
    d: Cow<'a, u32>,
    e: Rc<u32>,
    f: Arc<u32>,
    g: Cell<u32>,
}
