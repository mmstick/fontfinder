extern crate failure;
extern crate itertools;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

pub mod dirs;
pub mod fc_cache;
pub mod fonts;
pub mod html;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate horrorshow;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use std::io;

#[derive(Debug, Fail)]
pub enum FontError {
    #[fail(display = "error getting font directory")]
    FontDirectory,
    #[fail(display = "connection error: {}", why)]
    Connection { why: reqwest::Error },
    #[fail(display = "font family not found in font list")]
    FontNotFound,
    #[fail(display = "I/O error: {}", why)]
    File { why: io::Error },
}

impl From<reqwest::Error> for FontError {
    fn from(err: reqwest::Error) -> FontError { FontError::Connection { why: err } }
}

impl From<io::Error> for FontError {
    fn from(err: io::Error) -> FontError { FontError::File { why: err } }
}
