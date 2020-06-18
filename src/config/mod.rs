mod option_set;

pub mod basic;
pub mod profile;
pub mod server;

use serde_json::{json, Value};
use tinytemplate::format_unescaped;

pub fn format(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
    if let Value::Bool(b) = value {
        format_unescaped(&json!(if *b { 1 } else { 0 }), output)
    } else {
        format_unescaped(value, output)
    }
}

pub use basic::BasicConfig;
pub use option_set::OptionSet;
pub use profile::Profile;
pub use server::ServerConfig;
