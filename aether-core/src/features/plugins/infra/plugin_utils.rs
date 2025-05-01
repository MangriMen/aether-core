use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    core::LauncherState, features::settings::LocationInfo, shared::domain::SerializableCommand,
};

pub fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            "/cache".to_owned(),
            location_info.plugin_cache_dir(plugin_id),
        ),
        ("/instances".to_owned(), location_info.instances_dir()),
    ])
}

pub fn plugin_path_to_relative<I, T>(
    id: &str,
    path: &str,
    allowed_prefixes: I,
) -> crate::Result<PathBuf>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let prefix = allowed_prefixes
        .into_iter()
        .find(|prefix| path.starts_with(prefix.as_ref()))
        .ok_or_else(|| {
            crate::ErrorKind::PluginNotAllowedPathError(id.to_string(), path.to_string()).as_error()
        })?;

    let stripped = path.strip_prefix(prefix.as_ref()).unwrap_or(path);

    Ok(PathBuf::from(
        stripped.strip_prefix('/').unwrap_or(stripped),
    ))
}

pub fn get_first_segment(path: &str) -> &str {
    let mut indices = path.match_indices('/').skip(1);
    if let Some((idx, _)) = indices.next() {
        &path[..idx]
    } else {
        path
    }
}

pub fn plugin_path_to_host(id: &str, path: &str) -> crate::Result<PathBuf> {
    if !path.starts_with('#') {
        return Ok(PathBuf::from(path));
    }

    let state = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(LauncherState::get())
    })?;

    let cleaned_path_str = path.strip_prefix('#').unwrap_or(path);
    let cleaned_path_start_segment = get_first_segment(cleaned_path_str);

    let allowed_paths = get_default_allowed_paths(&state.locations, id);
    let base_dir = allowed_paths
        .get(cleaned_path_start_segment)
        .ok_or_else(|| {
            crate::ErrorKind::PluginNotAllowedPathError(
                id.to_string(),
                cleaned_path_str.to_string(),
            )
            .as_error()
        })?;

    if !base_dir.is_dir() {
        std::fs::create_dir_all(base_dir)?;
    }

    let stripped_path = plugin_path_to_relative(id, cleaned_path_str, allowed_paths.keys())?;
    let host_path = base_dir.join(stripped_path);

    let canonical_base = crate::shared::canonicalize(base_dir)?;
    let canonical_host = crate::shared::canonicalize(&host_path)?;

    if !canonical_host.starts_with(&canonical_base) {
        return Err(crate::ErrorKind::PluginNotAllowedPathError(
            id.to_string(),
            canonical_host.to_string_lossy().to_string(),
        )
        .as_error());
    }

    Ok(host_path)
}

pub fn plugin_path_to_host_from_path(id: &str, path: &Path) -> crate::Result<PathBuf> {
    plugin_path_to_host(id, path.to_string_lossy().as_ref())
}

pub fn plugin_command_to_host(
    id: &str,
    command: &SerializableCommand,
) -> crate::Result<SerializableCommand> {
    let fixed_program = plugin_path_to_host(id, &command.program)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| command.program.clone());

    let fixed_args: Vec<String> = command
        .args
        .iter()
        .map(|arg| plugin_path_to_host(id, arg).map(|p| p.to_string_lossy().to_string()))
        .collect::<crate::Result<_>>()?;

    let fixed_current_dir = command
        .current_dir
        .as_ref()
        .map(|current_dir| plugin_path_to_host_from_path(id, current_dir))
        .transpose()?;

    Ok(SerializableCommand {
        program: fixed_program,
        args: fixed_args,
        current_dir: fixed_current_dir,
    })
}

pub fn log_level_from_u32(level: u32) -> log::Level {
    match level {
        1 => log::Level::Error,
        2 => log::Level::Warn,
        3 => log::Level::Info,
        4 => log::Level::Debug,
        _ => log::Level::Trace,
    }
}
