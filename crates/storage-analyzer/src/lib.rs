use analyzer::{ContractAnalysis, TypeRef};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Invalid analysis input")]
    InvalidInput,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Confidence {
    Known,
    Inferred,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Durability {
    Persistent,
    Instance,
    Temporary,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StorageEntry {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
    pub durability: Durability,
    pub confidence: Confidence,
    pub namespace: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StorageAnalysis {
    pub entries: Vec<StorageEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FieldChange {
    Added { name: String, type_ref: TypeRef },
    Removed { name: String, type_ref: TypeRef },
    TypeChanged { name: String, old: TypeRef, new: TypeRef },
    Preserved { name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StorageDiff {
    pub changed_value_types: Vec<FieldChange>,
}

pub struct StorageAnalyzer;

impl StorageAnalyzer {
    pub fn analyze(contract: &ContractAnalysis) -> Result<StorageAnalysis, StorageError> {
        let mut entries = Vec::new();

        // Heuristic: Search for get_* and set_* methods that map an input to an output.
        // For migration_v1 / v2, we have:
        // pub fn get_record(env: Env, owner: Address) -> Option<Record>
        for func in &contract.functions {
            if func.name.starts_with("get_") {
                let namespace = func.name.strip_prefix("get_").unwrap_or("Unknown").to_string();
                
                // Usually first param is Env, wait! The WASM ABI doesn't show `Env`. 
                // The analyzer strips it if it's not actually an argument in the WASM signature or the XDR function signature.
                // In Soroban, `Env` is not present in `contractspecv0` function signature.
                // Let's assume the first remaining argument is the key.
                let key_type = if let Some(input) = func.inputs.first() {
                    input.type_ref.clone()
                } else {
                    TypeRef::Unknown
                };

                // The return type is typically `Option<T>` or `Result<T, E>`.
                let value_type = if let Some(output) = func.outputs.first() {
                    match output {
                        TypeRef::Complex(s) if s.starts_with("Option<") => {
                            let inner = s.trim_start_matches("Option<").trim_end_matches(">");
                            TypeRef::Complex(inner.to_string())
                        }
                        TypeRef::Complex(s) if s.starts_with("Result<") => {
                            let inner = s.trim_start_matches("Result<").split(',').next().unwrap_or("Unknown").trim();
                            TypeRef::Complex(inner.to_string())
                        }
                        _ => output.clone(),
                    }
                } else {
                    TypeRef::Unknown
                };

                entries.push(StorageEntry {
                    key_type,
                    value_type,
                    durability: Durability::Unknown,
                    confidence: Confidence::Inferred,
                    namespace,
                });
            }
        }

        Ok(StorageAnalysis { entries })
    }
}

impl StorageDiff {
    pub fn compare(old: &StorageAnalysis, new: &StorageAnalysis, old_types: &[analyzer::TypeAnalysis], new_types: &[analyzer::TypeAnalysis]) -> Self {
        let mut changed_value_types = Vec::new();

        for old_entry in &old.entries {
            if let Some(new_entry) = new.entries.iter().find(|e| e.namespace == old_entry.namespace) {
                // If the underlying type changed (e.g. Record -> RecordV2 or structural changes)
                // We resolve their underlying structural types
                let old_struct = resolve_struct(&old_entry.value_type, old_types);
                let new_struct = resolve_struct(&new_entry.value_type, new_types);

                if let (Some(old_fields), Some(new_fields)) = (old_struct, new_struct) {
                    for old_field in &old_fields {
                        if let Some(new_field) = new_fields.iter().find(|f| f.name == old_field.name) {
                            if old_field.type_ref != new_field.type_ref {
                                changed_value_types.push(FieldChange::TypeChanged {
                                    name: old_field.name.clone(),
                                    old: old_field.type_ref.clone(),
                                    new: new_field.type_ref.clone(),
                                });
                            } else {
                                changed_value_types.push(FieldChange::Preserved {
                                    name: old_field.name.clone(),
                                });
                            }
                        } else {
                            changed_value_types.push(FieldChange::Removed {
                                name: old_field.name.clone(),
                                type_ref: old_field.type_ref.clone(),
                            });
                        }
                    }

                    for new_field in &new_fields {
                        if !old_fields.iter().any(|f| f.name == new_field.name) {
                            changed_value_types.push(FieldChange::Added {
                                name: new_field.name.clone(),
                                type_ref: new_field.type_ref.clone(),
                            });
                        }
                    }
                }
            }
        }

        Self {
            changed_value_types,
        }
    }
}

fn resolve_struct(type_ref: &TypeRef, types: &[analyzer::TypeAnalysis]) -> Option<Vec<analyzer::FieldAnalysis>> {
    match type_ref {
        TypeRef::Complex(name) => {
            if let Some(t) = types.iter().find(|t| t.name == *name) {
                return Some(t.fields.clone());
            }
            None
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use analyzer::{Analyzer, ContractAnalysis, TypeRef};
    use std::fs;
    use std::path::PathBuf;

    fn get_fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../target/wasm32v1-none/release/");
        path.push(name);
        path.with_extension("wasm")
    }

    #[test]
    fn test_a_v1_storage_analysis() {
        let wasm = fs::read(get_fixture_path("migration_v1")).unwrap();
        let analysis = Analyzer::analyze(&wasm).unwrap();
        
        let storage = StorageAnalyzer::analyze(&analysis).unwrap();
        let record_entry = storage.entries.iter().find(|e| e.namespace == "record").expect("Missing record entry");
        
        assert_eq!(record_entry.key_type, TypeRef::Simple("Address".to_string()));
        assert_eq!(record_entry.value_type, TypeRef::Complex("Record".to_string()));
        assert_eq!(record_entry.confidence, Confidence::Inferred);
    }

    #[test]
    fn test_b_v2_storage_analysis() {
        let wasm = fs::read(get_fixture_path("migration_v2")).unwrap();
        let analysis = Analyzer::analyze(&wasm).unwrap();
        
        let storage = StorageAnalyzer::analyze(&analysis).unwrap();
        let record_entry = storage.entries.iter().find(|e| e.namespace == "record").expect("Missing record entry");
        
        assert_eq!(record_entry.key_type, TypeRef::Simple("Address".to_string()));
        assert_eq!(record_entry.value_type, TypeRef::Complex("RecordV2".to_string()));
        assert_eq!(record_entry.confidence, Confidence::Inferred);
    }

    #[test]
    fn test_c_storage_comparison() {
        let wasm_v1 = fs::read(get_fixture_path("migration_v1")).unwrap();
        let analysis_v1 = Analyzer::analyze(&wasm_v1).unwrap();
        let storage_v1 = StorageAnalyzer::analyze(&analysis_v1).unwrap();

        let wasm_v2 = fs::read(get_fixture_path("migration_v2")).unwrap();
        let analysis_v2 = Analyzer::analyze(&wasm_v2).unwrap();
        let storage_v2 = StorageAnalyzer::analyze(&analysis_v2).unwrap();

        let diff = StorageDiff::compare(&storage_v1, &storage_v2, &analysis_v1.types, &analysis_v2.types);
        
        // We expect version to be added, owner and value preserved
        let version_added = diff.changed_value_types.iter().any(|f| matches!(f, FieldChange::Added { name, .. } if name == "version"));
        assert!(version_added, "Missing version field addition");

        let owner_preserved = diff.changed_value_types.iter().any(|f| matches!(f, FieldChange::Preserved { name } if name == "owner"));
        assert!(owner_preserved, "Missing owner preservation");

        let value_preserved = diff.changed_value_types.iter().any(|f| matches!(f, FieldChange::Preserved { name } if name == "value"));
        assert!(value_preserved, "Missing value preservation");
    }

    #[test]
    fn test_d_unknown_information() {
        // Contract without any get_ methods shouldn't hallucinate storage
        let mock_contract = ContractAnalysis {
            has_metadata: false,
            has_env_metadata: false,
            env_metadata: None,
            metadata: None,
            functions: vec![],
            types: vec![],
        };
        let storage = StorageAnalyzer::analyze(&mock_contract).unwrap();
        assert!(storage.entries.is_empty());
    }

    #[test]
    fn test_e_serialization() {
        let storage = StorageAnalysis {
            entries: vec![StorageEntry {
                key_type: TypeRef::Simple("Address".to_string()),
                value_type: TypeRef::Complex("Record".to_string()),
                durability: Durability::Unknown,
                confidence: Confidence::Inferred,
                namespace: "record".to_string(),
            }],
        };
        
        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("\"Address\""));
        assert!(json.contains("\"Unknown\""));
        assert!(json.contains("\"Inferred\""));
    }
}
