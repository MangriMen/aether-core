// use aether_core::state::{LauncherState, Settings};
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

async fn init_launcher(_args: &Args) -> anyhow::Result<()> {
    // let current_dir = std::env::current_dir().unwrap();

    // let settings_dir = args.path.as_ref().unwrap_or(&current_dir);

    // LauncherState::init(&Settings {
    //     launcher_dir: Some(settings_dir.to_string_lossy().to_string()),
    //     metadata_dir: Some(settings_dir.to_string_lossy().to_string()),
    //     max_concurrent_downloads: 4,
    // })
    // .await?;

    Ok(())
}

async fn process_args(_args: &Args) -> anyhow::Result<()> {
    // match &args.command {
    //     SubCommands::Install(command) => {
    //         launch_minecraft(
    //             &Instance {
    //                 install_stage: aether_core::state::InstanceInstallStage::NotInstalled,
    //                 id: "Test".to_owned(),
    //                 path: "./test".to_owned(),
    //                 name: "Test".to_owned(),
    //                 icon_path: None,
    //                 game_version: command.version.to_owned(),
    //                 loader: aether_core::state::ModLoader::Vanilla,
    //                 loader_version: None,
    //                 java_path: None,
    //                 extra_launch_args: None,
    //                 custom_env_vars: None,
    //                 memory: None,
    //                 force_fullscreen: None,
    //                 game_resolution: None,
    //                 time_played: 0,
    //                 created: Utc::now(),
    //                 modified: Utc::now(),
    //                 last_played: None,
    //             },
    //             &[],
    //             &[],
    //             &MemorySettings { maximum: 1024 },
    //             &WindowSize(800, 600),
    //             &Credentials {
    //                 id: Uuid::new_v4(),
    //                 username: "Test".to_owned(),
    //                 access_token: "".to_owned(),
    //                 refresh_token: "".to_owned(),
    //                 expires: Utc::now().with_year(2025).unwrap(),
    //                 active: true,
    //             },
    //             None,
    //         )
    //         .await?
    //     }
    // };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::init();

    log::info!("Starting Aether Launcher cli");

    init_launcher(&args).await?;

    process_args(&args).await?;

    Ok(())
}
