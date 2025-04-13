use daedalus::modded;

use crate::shared::IOError;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
};

use super::get_lib_path;

pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, modded::SidedDataEntry>,
) -> crate::Result<Vec<String>> {
    let mut new_arguments = Vec::new();

    for argument in arguments {
        let trimmed_arg = &argument.as_ref()[1..argument.as_ref().len() - 1];
        if argument.as_ref().starts_with('{') {
            if let Some(entry) = data.get(trimmed_arg) {
                new_arguments.push(if entry.client.starts_with('[') {
                    get_lib_path(
                        libraries_path,
                        &entry.client[1..entry.client.len() - 1],
                        true,
                    )?
                } else {
                    entry.client.clone()
                })
            }
        } else if argument.as_ref().starts_with('[') {
            new_arguments.push(get_lib_path(libraries_path, trimmed_arg, true)?)
        } else {
            new_arguments.push(argument.as_ref().to_string())
        }
    }

    Ok(new_arguments)
}

pub async fn get_processor_main_class(path: String) -> crate::Result<Option<String>> {
    let main_class = tokio::task::spawn_blocking(move || {
        let zip_file = std::fs::File::open(&path).map_err(|e| IOError::with_path(e, &path))?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor at {}", path)).as_error()
        })?;

        let file = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor manifest at {}", path))
                .as_error()
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut line = line.map_err(IOError::from)?;
            line.retain(|c| !c.is_whitespace());

            if line.starts_with("Main-Class:") {
                if let Some(class) = line.split(':').nth(1) {
                    return Ok(Some(class.to_string()));
                }
            }
        }

        Ok::<Option<String>, crate::Error>(None)
    })
    .await??;

    Ok(main_class)
}
