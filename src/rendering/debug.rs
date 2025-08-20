// General purpose debugging functions

#[allow(dead_code)]
pub fn green_text(text: String) -> String {
    format!("\x1b[32m{text}\x1b[0m")
}

#[allow(dead_code)]
fn red_text(text: String) -> String {
    format!("\x1b[31m{text}\x1b[0m")
}

#[allow(dead_code)]
fn yellow_text(text: String) -> String {
    format!("\x1b[33m{text}\x1b[0m")
}
