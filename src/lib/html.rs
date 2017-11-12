use horrorshow::helper::doctype;
use itertools::sorted;

const FONT_URL: &str = "http://fonts.googleapis.com/css?family=";

fn get_font_url(family: &str) -> String { [FONT_URL, family].concat() }

/// Simply build a HTML page with the correct font family, at a given size, and with a
/// supplied string of text. Then pass the results into a specified closure.
pub fn generate<F: Fn(&str)>(
    family: &str,
    variants: &[String],
    size: f64,
    text: &str,
    dark: bool,
    closure: F,
) {
    let variants = sorted(variants.iter().map(|x| get_style(x)));
    let css = get_font_url(&[family, ":", &variants.join(",")].concat());
    let string = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    link(rel="stylesheet", href=&css) { }
                    style {
                        : format!("body {{ font-size: {}em; font-family: '{}'; }}", size, family);
                        : "html { margin: 0; border: 0; padding: 0 }";
                        : "p { margin: .5em }";
                        @ for variant in &variants {
                            @ if variant.ends_with('i') {
                                : format!(
                                    "#w{} {{ font-style: italic; font-weight: {}; }}",
                                    &variant,
                                    &variant[..variant.len()-1]
                                );
                            } else {
                                : format!("#w{0} {{ font-weight: {0}; }}", variant);
                            }
                        }
                    }
                    style {
                        @ if dark { : "html { background: #333; color: #FFF }" }
                    }
                }
                body {
                    @ for variant in &variants {
                        div(id=&["w", variant].concat()) {
                            h3 { : variant }
                            @ for line in text.lines() {
                                p { : line }
                            }
                        }
                    }
                }
            }
        }
    );

    closure(string.as_str());
}

/// Converts the font style provided by Google (regular, 300italic, italic, 500regular) to the
/// corresponding style that is accepted by their CSS API. Not sure why their API differs.
fn get_style(input: &str) -> &str {
    input.rfind(char::is_numeric).map_or_else(
        || if input == "italic" { "400i" } else { "400" },
        |pos| if &input[pos + 1..] == "italic" { &input[..pos + 2] } else { &input[..pos + 1] },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style() {
        let input = ["regular", "italic", "500italic", "500regular"];
        assert_eq!(
            input.iter().map(|x| get_style(x)).collect::<Vec<&str>>(),
            vec!["400", "400i", "500i", "500"]
        );
    }
}
