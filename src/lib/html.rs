use horrorshow::helper::doctype;

const FONT_URL: &str = "http://fonts.googleapis.com/css?family=";

fn get_font_url(family: &str) -> String { [FONT_URL, family].concat() }

/// Simply build a HTML page with the correct font family, at a given size, and with a
/// supplied string of text. Then pass the results into a specified closure.
pub fn generate<F: Fn(&str)>(family: &str, size: f64, text: &str, dark: bool, closure: F) {
    let string = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    link(rel="stylesheet", href=&get_font_url(family)) { }
                    style {
                        : format!("body {{ font-size: {}em; font-family: {}; }}", size, family);
                        : "html { margin: 0; border: 0; padding: 0 }";
                        : "p { margin: .5em }"
                    }
                    @ if dark {
                        style { : "html { background: #333; color: #FFF }" }
                    }
                }
                body {
                    @ for line in text.lines() {
                        p { : line }
                    }
                }
            }
        }
    );

    closure(string.as_str());
}
