use horrorshow::helper::doctype;

const FONT_URL: &str = "http://fonts.googleapis.com/css?family=";

fn get_font_url(family: &str) -> String { [FONT_URL, family].concat() }

/// Simply build a HTML page with the correct font family, at a given size, and with a
/// supplied string of text. Then pass the results into a specified closure.
pub fn generate<F: Fn(&str)>(family: &str, size: f64, text: &str, closure: F) {
    let string = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    link(rel="stylesheet", href=&get_font_url(family)) { }
                    style {
                        : format!("body {{ font-size: {}em; font-family: {}; }}", size, family)
                    }
                }
                body {
                    p {
                        : text;
                    }
                }
            }
        }
    );

    closure(string.as_str());
}
