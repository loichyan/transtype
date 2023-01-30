#![allow(unused)]

use transtype::pipe;

#[transtype::define]
#[derive1(Clone, Copy)]
#[derive2(Clone)]
#[derive(Debug)]
struct A {
    pub a: String,
    pub b: usize,
}

pipe! {
    A
    -> select_attr(
        derive1 => derive,
        derive2 => _,
        _,
    )
    -> select(b)
}
