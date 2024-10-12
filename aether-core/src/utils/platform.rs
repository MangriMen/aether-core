use daedalus::minecraft::Os;

#[cfg(target_pointer_width = "64")]
pub const ARCH_WIDTH: &str = "64";

#[cfg(target_pointer_width = "32")]
pub const ARCH_WIDTH: &str = "32";

// OS detection
pub trait OsExt {
    /// Get the OS of the current system
    fn native() -> Self;

    /// Gets the OS + Arch of the current system
    fn native_arch(java_arch: &str) -> Self;
}

impl OsExt for Os {
    fn native_arch(java_arch: &str) -> Self {
        if std::env::consts::OS == "windows" {
            if java_arch == "aarch64" {
                Os::WindowsArm64
            } else {
                Os::Windows
            }
        } else if std::env::consts::OS == "linux" {
            if java_arch == "aarch64" {
                Os::LinuxArm64
            } else if java_arch == "arm" {
                Os::LinuxArm32
            } else {
                Os::Linux
            }
        } else if std::env::consts::OS == "macos" {
            if java_arch == "aarch64" {
                Os::OsxArm64
            } else {
                Os::Osx
            }
        } else {
            Os::Unknown
        }
    }

    fn native() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Osx,
            "linux" => Self::Linux,
            _ => Self::Unknown,
        }
    }
}
