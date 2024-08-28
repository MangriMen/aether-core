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

fn init_launcher(args: &Args) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().unwrap();

    let settings_dir = args.path.as_ref().unwrap_or(&current_dir);

    Ok(())
}

async fn process_args(args: &Args) -> anyhow::Result<()> {
    match &args.command {
        SubCommands::Install(command) => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_launcher(&args)?;

    process_args(&args).await?;

    Ok(())
}
