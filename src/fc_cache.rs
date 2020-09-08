use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Used to signal the fc cache event loop to spawn an fc-cache process.
pub static RUN_FC_CACHE: AtomicBool = AtomicBool::new(false);

/// An event loop that waits for RUN_FC_CACHE to be set to true, and then executes `fc-cache -f`
/// to ensure that the font cache is updated after installing or removing a font family.
pub fn fc_cache_event_loop() {
    thread::spawn(|| {
        loop {
            // Try not to eat up CPU cycles by only checking RUN_FC_CACHE every 100ms.
            thread::sleep(Duration::from_millis(100));

            // If the UI has set the atomic value to true, this will spawn a fc-fache
            // process.
            if RUN_FC_CACHE.swap(false, Ordering::Relaxed) {
                eprintln!("fontfinder: execution fc-cache -f in the background");
                let _ = Command::new("fc-cache")
                    .arg("-f")
                    .spawn()
                    .map(|mut child| child.wait());
            }
        }
    });
}
