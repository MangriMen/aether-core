use std::path::Path;

use daedalus::minecraft;
use uuid::Uuid;

use crate::{
    features::{auth::Credentials, settings::WindowSize},
    shared::canonicalize,
};

use super::parse_arguments;

// Replaces the space separator with a newline character, as to not split the arguments
const TEMPORARY_REPLACE_CHAR: &str = "\n";

#[allow(clippy::too_many_arguments)]
pub fn get_minecraft_arguments(
    arguments: Option<&[minecraft::Argument]>,
    legacy_arguments: Option<&str>,
    credentials: &Credentials,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &minecraft::VersionType,
    resolution: WindowSize,
    java_arch: &str,
) -> crate::Result<Vec<String>> {
    if let Some(arguments) = arguments {
        let mut parsed_arguments = Vec::new();

        parse_arguments(
            arguments,
            &mut parsed_arguments,
            |arg| {
                parse_minecraft_argument(
                    arg,
                    &credentials.access_token,
                    &credentials.username,
                    credentials.id,
                    version,
                    asset_index_name,
                    game_directory,
                    assets_directory,
                    version_type,
                    resolution,
                )
            },
            java_arch,
        )?;

        Ok(parsed_arguments)
    } else if let Some(legacy_arguments) = legacy_arguments {
        let mut parsed_arguments = Vec::new();
        for x in legacy_arguments.split(' ') {
            parsed_arguments.push(parse_minecraft_argument(
                &x.replace(' ', TEMPORARY_REPLACE_CHAR),
                &credentials.access_token,
                &credentials.username,
                credentials.id,
                version,
                asset_index_name,
                game_directory,
                assets_directory,
                version_type,
                resolution,
            )?);
        }
        Ok(parsed_arguments)
    } else {
        Ok(Vec::new())
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_minecraft_argument(
    argument: &str,
    access_token: &str,
    username: &str,
    uuid: Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &minecraft::VersionType,
    resolution: WindowSize,
) -> crate::Result<String> {
    Ok(argument
        .replace("${accessToken}", access_token)
        .replace("${auth_access_token}", access_token)
        .replace("${auth_session}", access_token)
        .replace("${auth_player_name}", username)
        // TODO: add auth xuid eventually
        .replace("${auth_xuid}", "0")
        .replace("${auth_uuid}", &uuid.simple().to_string())
        .replace("${uuid}", &uuid.simple().to_string())
        .replace("${clientid}", "c4502edb-87c6-40cb-b595-64a280cf8906")
        .replace("${user_properties}", "{}")
        .replace("${user_type}", "msa")
        .replace("${version_name}", version)
        .replace("${assets_index_name}", asset_index_name)
        .replace(
            "${game_directory}",
            &canonicalize(game_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified game directory {} does not exist",
                        game_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${assets_root}",
            &canonicalize(assets_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${game_assets}",
            &canonicalize(assets_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace("${version_type}", version_type.as_str())
        .replace("${resolution_width}", &resolution.0.to_string())
        .replace("${resolution_height}", &resolution.1.to_string()))
}
