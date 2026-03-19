use super::Formatter;

pub struct FreeformFormatter;

impl Formatter for FreeformFormatter {
    fn name(&self) -> &str {
        "freeform"
    }

    fn format(&self, message: &str) -> String {
        message
            .trim()
            .trim_matches('`')
            .trim()
            .to_string()
    }
}