use {dirs, FontError};
use itertools::Itertools;
use reqwest::{self, Client};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

const API_KEY: &str = "AIzaSyDpvpba_5RvJSvmXEJS7gZDezDaMlVTo4c";

lazy_static! {
    static ref URL: String = {
        format!("https://www.googleapis.com/webfonts/v1/webfonts?key={}", API_KEY)
    };
}

/// The JSON response from Google that contains information on Google's font archive.
#[derive(Deserialize)]
pub struct FontsList {
    pub kind:  String,
    pub items: Vec<Font>,
}

impl FontsList {
    /// Downloads/installs each variant of a given font family.
    pub fn download<W>(&self, writer: &mut W, family: &str) -> Result<(), FontError>
        where W: Write
    {
        // Initialize a client that will be re-used between requests.
        let client = Client::new();

        // The base path of the local font directory will be used to construct future file paths.
        let path = dirs::font_cache().ok_or(FontError::FontDirectory)?;
        // Recursively creates the aforementioned path if it does not already exist.
        dirs::recursively_create(&path)?;

        // Finds the given font in the font list and return it's reference.
        let font = self.get_family(family).ok_or(FontError::FontNotFound)?;

        // Download/install each variant of the given font family.
        for (variant, uri) in &font.files {
            // Create a variant of the path with this variant's filename.
            let path = dirs::get_font_path(&path, family, &variant, &uri);

            // Writes information about what's happening to the UI's console.
            let _ = writer.write(format!("Installing '{:?}'\n", path).as_bytes());

            // GET the font variant from Google's servers.
            let mut data = client.get(uri.as_str()).send()?;

            // Then create that file for writing, and write the font's data to the file.
            let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
            io::copy(&mut data, &mut file)?;
        }

        Ok(())
    }

    /// Removes the installed font from the system.
    pub fn remove<W>(&self, writer: &mut W, family: &str) -> Result<(), FontError>
        where W: Write
    {
        // Get the base directory of the local font directory
        let path = dirs::font_cache().ok_or(FontError::FontDirectory)?;

        // Find the given font in the font list and return it's reference.
        let font = self.get_family(family).ok_or(FontError::FontNotFound)?;

        // Remove each variant of the given font family.
        for (variant, uri) in &font.files {
            // Create a variant of the path with this variant's filename.
            let path = dirs::get_font_path(&path, family, &variant, &uri);

            // Writes information about what's happening to the UI's console.
            let _ = writer.write(format!("Removing '{:?}'\n", path).as_bytes());

            // Then remove that file, if it exists.
            if let Err(why) = fs::remove_file(&path) {
                let msg = format!("Unable to remove '{:?}': {}\n", path, why);
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
        self.items.iter().map(|f| f.category.as_str()).unique().map(String::from).collect()
    }
}

/// A representation of an individual font within Google's font archive.
#[derive(Deserialize)]
pub struct Font {
    pub kind:     String,
    pub family:   String,
    pub category: String,
    pub variants: Vec<String>,
    pub subsets:  Vec<String>,
    pub version:  String,
    pub modified: Option<String>,
    pub files:    HashMap<String, String>,
}

/// Obtains the list of fonts from Google's font archive, whereby serde is automatically
/// deserializing the JSON into the `FontsList` structure.
pub fn obtain() -> reqwest::Result<FontsList> { reqwest::get(URL.as_str())?.json() }
