#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: Option<String>,
}

pipe! {
    A
    => wrap(Option)
}
