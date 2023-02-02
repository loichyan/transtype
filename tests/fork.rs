#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: usize,
}

pipe! {
    A
    -> fork(
        A={
            -> finish()
        }
        WrappedA={
            -> wrap(Option)
            -> wrapped(A)
            -> finish()
        }
    )
}
