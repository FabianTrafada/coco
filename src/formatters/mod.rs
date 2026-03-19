pub mod conventional;
pub mod freeform;

pub trait Formatter {
    fn name(&self) -> &str;
    fn format(&self, message: &str) -> String;
}

pub fn get_formatter(name: &str) -> Box<dyn Formatter> {
    match name {
        "conventional" => Box::new(conventional::ConventionalFormatter),
        _ => Box::new(freeform::FreeformFormatter),
    }
}