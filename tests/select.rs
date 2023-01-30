#![allow(unused)]

use transtype::pipe;

#[transtype::define]
#[derive1(Debug, Clone, Copy)]
#[derive2(Debug)]
struct A {
    pub a: String,
    pub b: usize,
}

pipe! {
    A
    -> select_attr(derive1 => derive)
    -> select(b)
}
