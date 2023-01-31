#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: usize,
}

pipe! {
    A
    -> finish()
}

pipe! {
    A
    -> rename(WrappedA)
    -> wrap(Option)
    -> save()
}

pipe! {
    WrappedA
    -> wrapped(A)
    -> finish()
}
