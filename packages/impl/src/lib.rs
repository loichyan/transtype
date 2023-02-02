use proc_macro::TokenStream;
use syn::{parse::Nothing, parse_macro_input};

#[proc_macro_attribute]
pub fn define(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);
    transtype_lib::private::define(input.into()).into()
}

#[proc_macro]
#[doc(hidden)]
pub fn __predefined(input: TokenStream) -> TokenStream {
    transtype_lib::private::predefined(input.into()).into()
}

macro_rules! expose_macros {
    ($($(#[$attr:meta])* $vis:vis fn $name:ident;)*) => {$(
        $(#[$attr])*
        #[proc_macro]
        $vis fn $name(input: TokenStream) -> TokenStream {
            ::transtype_lib::private::$name(input.into()).into()
        }
    )*};
}

expose_macros! {
    pub fn pipe;

    pub fn transform;

    /// Consumes all rest tokens, generates a macro prefixes with `DEBUG_` which
    /// returns the stringified tokens tree.
    pub fn debug;

    pub fn extend;

    /// Consumes all rest tokens and returns.
    pub fn finish;

    pub fn rename;

    pub fn save;

    pub fn select;

    pub fn select_attr;

    pub fn resume;

    pub fn wrap;

    pub fn wrapped;
}
