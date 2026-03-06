use cargo_metadata::MetadataCommand;

#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub repository: Option<String>,
}

pub fn get_project_dependencies() -> Result<Vec<DependencyInfo>, Box<dyn std::error::Error>> {
    let metadata = MetadataCommand::new().exec()?;

    let mut dependencies = Vec::new();

    // The metadata.packages list contains *all* crates in the lockfile.
    // metadata.workspace_members contains the ID of the local project itself.
    
    // We want to analyze all third-party crates, so we filter out workspace members.
    let workspace_members = metadata.workspace_members;

    for package in metadata.packages {
        if !workspace_members.contains(&package.id) {
            dependencies.push(DependencyInfo {
                name: package.name.to_string(),
                version: package.version.to_string(),
                repository: package.repository.clone(),
            });
        }
    }

    Ok(dependencies)
}
