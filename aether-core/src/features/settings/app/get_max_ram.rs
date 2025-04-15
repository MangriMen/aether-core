use sysinfo::System;

pub fn get_max_ram() -> u64 {
    System::new_all().total_memory()
}
