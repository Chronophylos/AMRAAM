use anyhow::{Context, Result};
use roxmltree::{Document, Node};
use std::{fs::File, io::prelude::*, path::Path};

#[derive(Debug)]
struct ModContainer<'a> {
    name: &'a str,
    from: &'a str,
    link: &'a str,
}

pub struct Mod<'a> {
    path: &'a Path,
    name: Option<&'a str>,
    id: Option<&'a str>,
}
pub struct Modpack<'a>(Vec<Mod<'a>>);

impl Modpack<'_> {
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let doc = Document::parse(buf.as_ref())?;

        let mod_containers = doc
            .descendants()
            .filter(|n| n.has_tag_name("tr"))
            .map(|n| {
                let vec = n
                    .children()
                    .filter(|n| n.has_tag_name("td"))
                    .collect::<Vec<Node>>();
                Ok(ModContainer {
                    name: vec
                        .get(0)
                        .context("Missing node")?
                        .text()
                        .context("Attribute has no text")?,
                    from: vec
                        .get(1)
                        .context("Missing node")?
                        .first_element_child()
                        .context("Node has no children")?
                        .text()
                        .context("Missing text")?,
                    link: vec
                        .get(2)
                        .context("Missing node")?
                        .first_element_child()
                        .context("Node has no children")?
                        .text()
                        .context("Missing text")?,
                })
            })
            .collect::<Result<Vec<ModContainer>>>()?;

        dbg!(mod_containers);

        unimplemented!()
    }
}
