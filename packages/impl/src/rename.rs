use syn::{Ident, Result};
use transtype_lib::{Command, CommandOutput};

pub struct Rename;

impl Command for Rename {
    type Args = Ident;

    fn execute(mut data: syn::DeriveInput, name: Self::Args) -> Result<CommandOutput> {
        data.ident = name;
        Ok(CommandOutput::Piped(data))
    }
}
