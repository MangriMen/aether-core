#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct Hooks {
    pub pre_launch: Option<String>,
    pub wrapper: Option<String>,
    pub post_exit: Option<String>,
}
