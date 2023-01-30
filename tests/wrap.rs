#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: usize,
}

pipe! {
    A
}

pipe! {
    A
    -> rename(WrappedA)
    -> wrap(Option from A)
}
