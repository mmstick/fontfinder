#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate horrorshow;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

pub mod dirs;
pub mod fonts;
pub mod html;

use async_process::Command;

pub async fn run_fc_cache() {
    let _ = Command::new("fc-cache").arg("-f").status().await;
}
