use cargo_metadata::MetadataCommand;

#[derive(Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
}

pub fn get_project_dependencies() -> Result<Vec<DependencyInfo>, Box<dyn std::error::Error>> {
    let metadata = MetadataCommand::new().exec()?;
    Ok(parse_dependencies(&metadata))
}

pub fn parse_dependencies(metadata: &cargo_metadata::Metadata) -> Vec<DependencyInfo> {
    let mut dependencies = Vec::new();
    let workspace_members = &metadata.workspace_members;
    let mut direct_dependency_ids = std::collections::HashSet::new();

    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            if workspace_members.contains(&node.id) {
                for dep in &node.dependencies {
                    direct_dependency_ids.insert(dep);
                }
            }
        }
    }

    for package in &metadata.packages {
        if direct_dependency_ids.contains(&package.id) {
            dependencies.push(DependencyInfo {
                name: package.name.to_string(),
                version: package.version.to_string(),
            });
        }
    }

    dependencies
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_dependencies() {
        let metadata_json = json!({
            "packages": [
                {
                    "name": "direct-dep",
                    "version": "1.0.0",
                    "id": "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
                    "dependencies": [],
                    "targets": [],
                    "features": {},
                    "manifest_path": "/path/to/direct/Cargo.toml"
                },
                {
                    "name": "transitive-dep",
                    "version": "0.5.0",
                    "id": "transitive-dep 0.5.0 (registry+https://github.com/rust-lang/crates.io-index)",
                    "dependencies": [],
                    "targets": [],
                    "features": {},
                    "manifest_path": "/path/to/transitive/Cargo.toml"
                },
                {
                    "name": "my-workspace-pkg",
                    "version": "0.1.0",
                    "id": "my-workspace-pkg 0.1.0 (path+file:///path/to/my-pkg)",
                    "dependencies": [],
                    "targets": [],
                    "features": {},
                    "manifest_path": "/path/to/my-pkg/Cargo.toml"
                }
            ],
            "workspace_members": [
                "my-workspace-pkg 0.1.0 (path+file:///path/to/my-pkg)"
            ],
            "resolve": {
                "nodes": [
                    {
                        "id": "my-workspace-pkg 0.1.0 (path+file:///path/to/my-pkg)",
                        "dependencies": [
                            "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"
                        ],
                        "deps": [
                            {
                                "name": "direct-dep",
                                "pkg": "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"
                            }
                        ]
                    },
                    {
                        "id": "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
                        "dependencies": [
                            "transitive-dep 0.5.0 (registry+https://github.com/rust-lang/crates.io-index)"
                        ],
                        "deps": [
                            {
                                "name": "transitive-dep",
                                "pkg": "transitive-dep 0.5.0 (registry+https://github.com/rust-lang/crates.io-index)"
                            }
                        ]
                    }
                ]
            },
            "target_directory": "/path/to/target",
            "version": 1,
            "workspace_root": "/path/to/root"
        });

        let metadata: cargo_metadata::Metadata = serde_json::from_value(metadata_json).unwrap();
        let deps = parse_dependencies(&metadata);

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "direct-dep");
        assert_eq!(deps[0].version, "1.0.0");
    }
}
