use crate::modpack::Modpack;
use std::path::Path;

pub struct Arma<'a> {
    path: &'a Path,
    profile: &'a Path,
    basic: &'a Path,
    server: &'a Path,
    modpack: Modpack,
    sever_modpack: Modpack,
}
