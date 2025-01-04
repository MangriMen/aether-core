pub enum MinecraftEvent {
    Loading,
    Process,
}

impl MinecraftEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            MinecraftEvent::Loading => "loading",
            MinecraftEvent::Process => "process",
        }
    }
}
