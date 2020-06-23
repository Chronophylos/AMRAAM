use crate::commands::prelude::*;

pub fn cli() -> App {
    SubCommand::with_name("fix")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    unimplemented!()
}
