#[cfg(target_os = "windows")]
pub const JAVA_BIN: &str = "java.exe";

#[cfg(not(target_os = "windows"))]
pub const JAVA_NO_WINDOW_BIN: &str = "java";

#[cfg(target_os = "windows")]
pub const JAVA_WINDOW_BIN: &str = "javaw.exe";

#[cfg(not(target_os = "windows"))]
pub const JAVA_BIN: &str = "java";
