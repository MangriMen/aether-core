use aether_core::{
    launcher::install_minecraft,
    state::{Instance, LauncherState, Settings},
};
use chrono::Utc;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct InstallCommand {
    version: String,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Install(InstallCommand),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    path: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: SubCommands,
}

async fn init_launcher(args: &Args) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().unwrap();

    let settings_dir = args.path.as_ref().unwrap_or(&current_dir);

    LauncherState::init(&Settings {
        launcher_dir: Some(settings_dir.to_string_lossy().to_string()),
        metadata_dir: Some(settings_dir.to_string_lossy().to_string()),
    })
    .await?;

    Ok(())
}

async fn process_args(args: &Args) -> anyhow::Result<()> {
    let state = LauncherState::get().await?;

    match &args.command {
        SubCommands::Install(command) => {
            install_minecraft(
                &Instance {
                    install_stage: aether_core::state::InstanceInstallStage::NotInstalled,
                    path: state.locations.metadata_dir().to_str().unwrap().to_owned(),
                    name: "Test".to_owned(),
                    icon_path: None,
                    game_version: command.version.to_owned(),
                    loader: aether_core::state::ModLoader::Vanilla,
                    loader_version: None,
                    java_path: None,
                    extra_launch_args: None,
                    custom_env_vars: None,
                    memory: None,
                    force_fullscreen: None,
                    game_resolution: None,
                    time_played: 0,
                    created: Utc::now(),
                    modified: Utc::now(),
                    last_played: None,
                },
                false,
            )
            .await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_launcher(&args).await?;

    process_args(&args).await?;

    Ok(())
}
