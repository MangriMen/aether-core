use crate::shared::OsExt;

pub fn get_classpath_separator(java_arch: &str) -> &'static str {
    use daedalus::minecraft::Os;

    match Os::native_arch(java_arch) {
        Os::Osx | Os::OsxArm64 | Os::Linux | Os::LinuxArm32 | Os::LinuxArm64 | Os::Unknown => ":",
        Os::Windows | Os::WindowsArm64 => ";",
    }
}
