use anyhow::{ensure, Context, Result};
use roxmltree::{Document, Node};
use std::{fs::File, io::prelude::*};

#[derive(Debug)]
struct ModContainer<'a> {
    name: &'a str,
    from: &'a str,
    link: &'a str,
}

#[derive(Clone)]
pub struct Mod {
    pub path: String,
    pub name: String,
    pub id: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct ModpackConfig {
    inherit: Option<Vec<String>>,
    mods: Option<Vec<String>>,
    path: Option<String>,
    url: Option<String>,
}

impl ModpackConfig {
    pub fn load_path(&self) -> Result<Vec<Mod>> {
        ensure!(self.path.is_some(), "Path to html file is not set");
        let path = self.path.clone().unwrap();

        let mut file = File::open(path).context("Could not open html file")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .context("Could not read html file")?;
        let doc = Document::parse(buf.as_ref()).context("Could not parse html document")?;

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
            .collect::<Result<Vec<ModContainer>>>()
            .context("Could not load ModContainers from html document")?;

        mod_containers
            .iter()
            .map(|container| {
                let id = container.link.split("=").last().context("Missing id")?;
                let path = format!("mods/{}", id);

                Ok(Mod {
                    path,
                    name: container.name.to_owned(),
                    id: Some(id.to_owned()),
                })
            })
            .collect()
    }

    pub fn load_url(self) -> Result<Vec<Mod>> {
        todo!()
    }

    pub fn as_modpack(&self) -> Result<Modpack> {
        let mods = self
            .mods
            .clone()
            // create empty vector if there are no strings
            .unwrap_or(Vec::new())
            .into_iter()
            // create Mod from String
            .map(|s| Mod {
                path: format!("mods/{}", &s),
                name: s,
                id: None,
            })
            .chain(
                // get mods from load_path if there are any
                if self.path.is_some() {
                    self.load_path().context("Could not load mods from file")?
                } else {
                    Vec::new()
                }
                .into_iter(),
            )
            .chain(
                // get mods from load_url if there are any
                if self.url.is_some() {
                    self.clone()
                        .load_url()
                        .context("Could not load mods from url")?
                } else {
                    Vec::new()
                }
                .into_iter(),
            )
            .collect();

        Ok(Modpack {
            mods,
            inherit: self.inherit.clone().unwrap_or(Vec::new()),
        })
    }
}

pub struct Modpack {
    pub mods: Vec<Mod>,
    pub inherit: Vec<String>,
}
