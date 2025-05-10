use std::path::PathBuf;

pub enum FileEventKind {
    Create,
    Modify,
    Remove,
}

pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: PathBuf,
}

// pub struct InstanceFileEvent {
//     pub instance_id: String,
//     pub event_type: InstanceEventType,
// }
