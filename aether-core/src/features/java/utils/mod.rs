pub mod check;
pub mod platform;

#[cfg(test)]
pub mod tests;

pub use check::check_jre_at_filepath;
pub use platform::get_classpath_separator;
