use super::Formatter;

pub struct ConventionalFormatter;

impl Formatter for ConventionalFormatter {
    fn name(&self) -> &str {
        "conventional"
    }

    fn format(&self, message: &str) -> String {
        // LLM sudah di-prompt untuk conventional format,
        // di sini kita clean up saja kalau ada artefak
        message
            .trim()
            .trim_matches('`')
            .trim()
            .to_string()
    }
}