use crate::commands::prelude::*;

pub fn cli() -> App {
    SubCommand::with_name("list")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    unimplemented!()
}
