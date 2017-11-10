use {dirs, FontError};
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

        // Get the base directory of the local font directory
        let path = dirs::font_cache().ok_or(FontError::FontDirectory)?;
        // Find the given font in the font list and return it's reference.
        let font = self.get_family(family).ok_or(FontError::FontNotFound)?;

        // Download/install each variant of the given font family.
        for (variant, uri) in &font.files {
            // Create a variant of the path with this variant's filename.
            let path = dirs::get_font_path(&path, family, &variant, &uri);
            // Write the action to stderr + the provided writer.
            let message = format!("Installing '{:?}'\n", path);
            eprint!("fontfinder: {}", message);
            let _ = writer.write(message.as_bytes());
            // Then create that file for writing.
            let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
            // GET the font variant from Google's servers.
            let mut data = client.get(uri.as_str()).send()?;
            // Copy the font's data directly to the initialized file.
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
            // Write the action to stderr + the provided writer.
            let message = format!("Removing '{:?}'\n", path);
            eprint!("fontfinder: {}", message);
            let _ = writer.write(message.as_bytes());
            // Then remove that file, if it exists.
            if let Err(why) = fs::remove_file(&path) {
                let message = format!("Unable to remove '{:?}': {}\n", path, why);
                eprint!("fontfinder: {}", message);
                let _ = writer.write(message.as_bytes());
            }
        }

        Ok(())
    }

    /// Obtain a reference to the given font family's information, if it exists.
    pub fn get_family<'a>(&'a self, family: &str) -> Option<&'a Font> {
        self.items
            .iter()
            .find(|&font| font.family.as_str() == family)
    }

    /// Sift through the font list and collect all unique categories found.
    pub fn get_categories(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::with_capacity(12);
        for font in &self.items {
            if !output.contains(&font.category) {
                output.push(font.category.to_owned());
            }
        }
        output
    }
}

pub struct FontVariant<'a> {
    pub variant: &'a str,
    pub uri:     &'a str,
}

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

pub fn obtain() -> reqwest::Result<FontsList> { reqwest::get(URL.as_str())?.json() }
