pub mod check;
pub mod platform;

#[cfg(test)]
pub mod __tests__;

pub use check::construct_java_from_jre;
pub use platform::get_classpath_separator;
