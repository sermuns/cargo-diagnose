use clap::{Parser, Subcommand};

mod api;
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
    // Both octocrab and reqwest use rustls under the hood. Since multiple crates bring in
    // aws-lc-rs and ring, we need to explicitly initialize one of the crypto providers.
    let _ = rustls::crypto::ring::default_provider().install_default();

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
            }

            let client = reqwest::Client::new();
            
            // Temporary output for Phase 3 verification
            for dep in &dependencies {
                if !json {
                    println!("- {} v{}", dep.name, dep.version);
                }

                // Query OSV for vulnerabilities
                match api::osv::check_vulnerabilities(&client, &dep.name, &dep.version).await {
                    Ok(osv_res) => {
                        if let Some(vulns) = osv_res.vulns {
                            if !json {
                                println!("  ⚠️ Found {} vulnerabilities!", vulns.len());
                                for v in vulns {
                                    println!("    - {}: {}", v.id, v.summary.unwrap_or_default());
                                }
                            }
                        } else {
                            if !json {
                                println!("  ✅ No known vulnerabilities");
                            }
                        }
                    }
                    Err(e) => {
                        if !json {
                            eprintln!("  ❌ Failed to fetch OSV data: {}", e);
                        }
                    }
                }

                // Query Crates.io for latest version / metadata
                match api::crates_io::get_crate_info(&client, &dep.name).await {
                    Ok(crates_res) => {
                        if !json {
                            if crates_res.crate_data.max_version != dep.version {
                                println!(
                                    "   Update available: {} -> {}",
                                    dep.version, crates_res.crate_data.max_version
                                );
                            } else {
                                println!("   Up to date");
                            }
                            
                            if let Some(repo) = crates_res.crate_data.repository {
                                println!("   Repo: {}", repo);
                                
                                // Query GitHub if it's a github repo
                                match api::github::get_repo_stats(&repo).await {
                                    Ok(Some(stats)) => {
                                        if !json {
                                            println!("     Stars: {}", stats.stars);
                                            println!("     Open Issues: {}", stats.open_issues);
                                            if stats.is_archived {
                                                println!("     ARCHIVED");
                                            }
                                        }
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        if !json {
                                            eprintln!("     Failed to fetch GitHub stats: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if !json {
                            eprintln!("Failed to fetch Crates.io data: {}", e);
                        }
                    }
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
