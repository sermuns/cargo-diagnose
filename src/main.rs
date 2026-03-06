use clap::{Parser, Subcommand};

mod metadata;

#[derive(Parser, Debug)]
#[command(name = "cargo-health")]
#[command(about = "Cargo Dependency Health Analyzer CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Runs a full dependency audit
    Analyze {
        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Fail if health score is below threshold
        #[arg(long)]
        fail_under: Option<u8>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { json, fail_under } => {
            if !json {
                println!("Scanning project...");
            }
            
            let dependencies = match metadata::get_project_dependencies() {
                Ok(deps) => deps,
                Err(e) => {
                    eprintln!("Failed to parse Cargo metadata: {}", e);
                    std::process::exit(1);
                }
            };
            
            if !json {
                println!("Found {} third-party dependencies.", dependencies.len());
                println!("Analyzing dependencies...");
                
                // Temporary output for Phase 2 verification
                for dep in &dependencies {
                    println!("- {} v{}", dep.name, dep.version);
                }
            }
            
            // Placeholder: output matching the goal score bounds.
            if let Some(threshold) = fail_under {
                if !json {
                    println!("Checking if project score is under threshold: {}", threshold);
                }
            }
        }
    }
    
    Ok(())
}
