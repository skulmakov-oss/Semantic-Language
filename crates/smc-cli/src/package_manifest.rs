use std::error::Error;
use std::fmt;

pub const PACKAGE_MANIFEST_BASELINE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRoot {
    pub manifest_dir: String,
    pub module_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageIdentity {
    pub name: String,
    pub root: PackageRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageDependencySource {
    LocalPath { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDependency {
    pub alias: String,
    pub package_name: String,
    pub source: PackageDependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub format_version: u32,
    pub package: PackageIdentity,
    pub dependencies: Vec<PackageDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManifestValidationCode {
    UnsupportedFormatVersion,
    EmptyPackageName,
    EmptyManifestDir,
    EmptyModuleRoot,
    EmptyDependencyAlias,
    DuplicateDependencyAlias,
    EmptyDependencyPackageName,
    EmptyLocalDependencyPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifestValidationError {
    pub code: PackageManifestValidationCode,
    pub message: String,
}

impl fmt::Display for PackageManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package manifest validation error: {}", self.message)
    }
}

impl Error for PackageManifestValidationError {}

impl PackageManifest {
    pub fn new(package: PackageIdentity, dependencies: Vec<PackageDependency>) -> Self {
        Self {
            format_version: PACKAGE_MANIFEST_BASELINE_VERSION,
            package,
            dependencies,
        }
    }
}

pub fn validate_package_manifest_baseline(
    manifest: &PackageManifest,
) -> Result<(), PackageManifestValidationError> {
    if manifest.format_version != PACKAGE_MANIFEST_BASELINE_VERSION {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::UnsupportedFormatVersion,
            message: format!(
                "unsupported package manifest format version {}; expected {}",
                manifest.format_version, PACKAGE_MANIFEST_BASELINE_VERSION
            ),
        });
    }

    if manifest.package.name.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyPackageName,
            message: "package name must not be empty".to_string(),
        });
    }

    if manifest.package.root.manifest_dir.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyManifestDir,
            message: "package manifest_dir must not be empty".to_string(),
        });
    }

    if manifest.package.root.module_root.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyModuleRoot,
            message: "package module_root must not be empty".to_string(),
        });
    }

    let mut seen_aliases = std::collections::BTreeSet::new();
    for dependency in &manifest.dependencies {
        if dependency.alias.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyDependencyAlias,
                message: "package dependency alias must not be empty".to_string(),
            });
        }
        if !seen_aliases.insert(dependency.alias.as_str()) {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::DuplicateDependencyAlias,
                message: format!(
                    "duplicate package dependency alias '{}'",
                    dependency.alias
                ),
            });
        }
        if dependency.package_name.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyDependencyPackageName,
                message: "package dependency package_name must not be empty".to_string(),
            });
        }
        match &dependency.source {
            PackageDependencySource::LocalPath { path } if path.trim().is_empty() => {
                return Err(PackageManifestValidationError {
                    code: PackageManifestValidationCode::EmptyLocalDependencyPath,
                    message: format!(
                        "package dependency '{}' requires a non-empty local path",
                        dependency.alias
                    ),
                });
            }
            PackageDependencySource::LocalPath { .. } => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package_root() -> PackageRoot {
        PackageRoot {
            manifest_dir: ".".to_string(),
            module_root: "src".to_string(),
        }
    }

    #[test]
    fn new_manifest_uses_canonical_baseline_version() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            Vec::new(),
        );
        assert_eq!(manifest.format_version, PACKAGE_MANIFEST_BASELINE_VERSION);
    }

    #[test]
    fn validate_package_manifest_accepts_local_path_dependency_inventory() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            vec![
                PackageDependency {
                    alias: "math".to_string(),
                    package_name: "math".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../math".to_string(),
                    },
                },
                PackageDependency {
                    alias: "ui".to_string(),
                    package_name: "ui".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../ui".to_string(),
                    },
                },
            ],
        );
        assert_eq!(
            manifest
                .dependencies
                .iter()
                .map(|dep| dep.alias.as_str())
                .collect::<Vec<_>>(),
            vec!["math", "ui"]
        );
        validate_package_manifest_baseline(&manifest).expect("valid local path manifest");
    }

    #[test]
    fn validate_package_manifest_rejects_duplicate_dependency_alias() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            vec![
                PackageDependency {
                    alias: "shared".to_string(),
                    package_name: "first".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../first".to_string(),
                    },
                },
                PackageDependency {
                    alias: "shared".to_string(),
                    package_name: "second".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../second".to_string(),
                    },
                },
            ],
        );
        let err = validate_package_manifest_baseline(&manifest).expect_err("must reject");
        assert_eq!(
            err.code,
            PackageManifestValidationCode::DuplicateDependencyAlias
        );
    }

    #[test]
    fn validate_package_manifest_rejects_empty_package_root_fields() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: PackageRoot {
                    manifest_dir: "".to_string(),
                    module_root: "src".to_string(),
                },
            },
            Vec::new(),
        );
        let err = validate_package_manifest_baseline(&manifest).expect_err("must reject");
        assert_eq!(err.code, PackageManifestValidationCode::EmptyManifestDir);
    }
}
