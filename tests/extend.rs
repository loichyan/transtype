#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: Option<String>,
}

#[transtype::define]
struct B {
    pub c: usize,
    pub d: Option<usize>,
}

pipe! {
    A
    -> extend(B)
    -> finish()
}
