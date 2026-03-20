use clap::Parser;

#[derive(Parser)]
#[command(
    name = "coco",
    about = "Generate commit message via CLI",
    version
)]
pub struct Cli {
    #[arg(short = 'y', long = "always-trust")]
    pub always_trust: bool,
    
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,
    
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
}