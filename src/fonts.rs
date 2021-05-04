use crate::dirs;
use crate::fl;
use anyhow::Context;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

const API_KEY: &str = "AIzaSyDpvpba_5RvJSvmXEJS7gZDezDaMlVTo4c";

lazy_static! {
    static ref URL_ALPHA: String = {
        format!(
            "https://www.googleapis.com/webfonts/v1/webfonts?sort=alpha&key={}",
            API_KEY
        )
    };
}

lazy_static! {
    static ref URL_DATE: String = {
        format!(
            "https://www.googleapis.com/webfonts/v1/webfonts?sort=date&key={}",
            API_KEY
        )
    };
}

lazy_static! {
    static ref URL_POPULARITY: String = {
        format!(
            "https://www.googleapis.com/webfonts/v1/webfonts?sort=popularity&key={}",
            API_KEY
        )
    };
}

lazy_static! {
    static ref URL_TRENDING: String = {
        format!(
            "https://www.googleapis.com/webfonts/v1/webfonts?sort=trending&key={}",
            API_KEY
        )
    };
}

/// The JSON response from Google that contains information on Google's font
/// archive.
#[derive(Deserialize)]
pub struct FontsList {
    pub kind: String,
    pub items: Vec<Font>,
}

impl FontsList {
    /// Downloads/installs each variant of a given font family.
    pub fn download<W>(&self, writer: &mut W, family: &str) -> anyhow::Result<()>
    where
        W: Write,
    {
        // The base path of the local font directory will be used to construct future
        // file paths.
        let path = dirs::font_cache()
            .with_context(|| fl!("error-font-directory"))?;

        // Recursively creates the aforementioned path if it does not already exist.
        dirs::recursively_create(&path)?;

        // Finds the given font in the font list and return it's reference.
        let font = self
            .get_family(family)
            .with_context(|| fl!("error-font-not-found"))?;

        // Download/install each variant of the given font family.
        for (variant, uri) in &font.files {
            // Create a variant of the path with this variant's filename.
            let path = dirs::get_font_path(&path, family, &variant, &uri);

            // Writes information about what's happening to the UI's console.
            let _ = writer.write(format!("{}\n", fl!("info-installing", path = format!("{:?}", path))).as_bytes());

            // GET the font variant from Google's servers.
            match ureq::get(uri.as_str()).call() {
                Ok(data) => {
                    // Then create that file for writing, and write the font's data to the file.
                    let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
                    io::copy(&mut data.into_reader(), &mut file)?;
                }
                Err(error) => {
                    return Err(anyhow!("{}", error)).with_context(|| fl!("error-fetch-failed"));
                }
            }
        }

        Ok(())
    }

    /// Removes the installed font from the system.
    pub fn remove<W>(&self, writer: &mut W, family: &str) -> anyhow::Result<()>
    where
        W: Write,
    {
        // Get the base directory of the local font directory
        let path = dirs::font_cache().with_context(|| fl!("error-font-directory"))?;

        // Find the given font in the font list and return it's reference.
        let font = self
            .get_family(family)
            .with_context(|| fl!("error-font-not-found"))?;

        // Remove each variant of the given font family.
        for (variant, uri) in &font.files {
            // Create a variant of the path with this variant's filename.
            let path = dirs::get_font_path(&path, family, &variant, &uri);

            let path_str = format!("{:?}", path);

            // Writes information about what's happening to the UI's console.
            let _ = writer.write(format!("{}\n", fl!("info-removing", path = path_str.clone())).as_bytes());

            // Then remove that file, if it exists.
            if let Err(why) = fs::remove_file(&path) {
                let msg = format!("{}: {}\n", fl!("error-unable-to-remove", path = path_str), why);
                let _ = writer.write(msg.as_bytes());
            }
        }

        Ok(())
    }

    /// Obtain a reference to the given font family's information, if it exists.
    pub fn get_family<'a>(&'a self, family: &str) -> Option<&'a Font> {
        self.items.iter().find(|f| f.family.as_str() == family)
    }

    /// Sift through the font list and collect all unique categories found.
    pub fn get_categories(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|f| f.category.as_str())
            .unique()
            .map(String::from)
            .collect()
    }
}

/// A representation of an individual font within Google's font archive.
#[derive(Deserialize)]
pub struct Font {
    pub kind: String,
    pub family: String,
    pub category: String,
    pub variants: Vec<String>,
    pub subsets: Vec<String>,
    pub version: String,
    pub modified: Option<String>,
    pub files: HashMap<String, String>,
}

pub enum Sorting {
    Alphabetical,
    DateAdded,
    Popular,
    Trending,
}

/// Obtains the list of fonts from Google's font archive, whereby serde is automatically
/// deserializing the JSON into the `FontsList` structure.
pub fn obtain(sort_by: Sorting) -> anyhow::Result<FontsList> {
    let url: &'static str = match sort_by {
        Sorting::Alphabetical => URL_ALPHA.as_str(),
        Sorting::DateAdded => URL_DATE.as_str(),
        Sorting::Popular => URL_POPULARITY.as_str(),
        Sorting::Trending => URL_TRENDING.as_str(),
    };

    match ureq::get(url).call() {
        Ok(resp) => {
            resp.into_json::<FontsList>()
                .with_context(|| fl!("error-deserialize"))
        },
        Err(error) => {
            Err(anyhow!("{}", error)).with_context(|| fl!("error-fetch-failed"))
        }
    }
}
