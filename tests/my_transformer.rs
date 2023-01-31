#![allow(unused)]

use transtype::pipe;

#[transtype::define]
struct A {
    pub a: String,
    pub b: usize,
}

macro_rules! my_transformer {
    (
        data={struct $name:ident $body:tt}
        args={$hello:literal}
        rest=$rest:tt
    ) => {
        ::transtype::transform! {
            data={}
            args={
                + {
                    struct $name $body
                    impl $name {
                        fn hello(&self) {
                            println!($hello, stringify!($name));
                        }
                    }
                }
            }
            rest=$rest
        }
    };
}

pipe! {
    A
    -> my_transformer("Hello, I'm {}!")
}
