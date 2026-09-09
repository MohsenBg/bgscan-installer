mod archive;
mod downloader;
mod installer;
mod net;
mod progress;
mod ui;

use clap::{Parser, Subcommand};

use crate::installer::Installer;
use crate::ui::{TerminalUI, UI};

#[derive(Parser, Debug)]
#[command(name = "bgscan-installer", about = "Installer for bgscan")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Version,

    Install {
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
}

// Release builds override this with APP_VERSION=<tag> (see scripts/build.sh);
// dev builds fall back to the Cargo package version.
pub const VERSION: &str = match option_env!("APP_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

fn main() {
    let cli = Cli::parse();
    let mut tui = TerminalUI::new(std::io::stdout());

    match cli.command {
        Commands::Version => {
            println!("bgscan {}", VERSION);
        }
        Commands::Install { version } => {
            tui.raw("\n");
            tui.brand();
            tui.muted(format!(" bgscan-installer • v{}", VERSION).as_str());
            tui.divider();

            if let Err(e) = run_install(&version) {
                tui.error(&e.to_string());
                std::process::exit(-1)
            }
        }
    }
}

pub fn run_install(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let installer = Installer::current()?;
    installer.install(version)?;
    Ok(())
}
