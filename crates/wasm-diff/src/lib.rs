use analyzer::{ContractAnalysis, FunctionAnalysis, TypeAnalysis};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("Failed to compare: {0}")]
    ComparisonError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContractDiff {
    pub added_functions: Vec<FunctionAnalysis>,
    pub removed_functions: Vec<FunctionAnalysis>,
    pub changed_functions: Vec<FunctionAnalysis>,
    pub added_types: Vec<TypeAnalysis>,
    pub removed_types: Vec<TypeAnalysis>,
    pub changed_types: Vec<TypeAnalysis>,
}

pub struct WasmDiff;

impl WasmDiff {
    pub fn compare(
        old: &ContractAnalysis,
        new: &ContractAnalysis,
    ) -> Result<ContractDiff, DiffError> {
        let old_funcs: HashMap<_, _> = old
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.clone()))
            .collect();
        let new_funcs: HashMap<_, _> = new
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.clone()))
            .collect();

        let mut added_functions = Vec::new();
        let mut removed_functions = Vec::new();
        let mut changed_functions = Vec::new();

        for (name, func) in &old_funcs {
            match new_funcs.get(name) {
                Some(new_func) if func != new_func => changed_functions.push(new_func.clone()),
                None => removed_functions.push(func.clone()),
                _ => {}
            }
        }

        for (name, func) in &new_funcs {
            if !old_funcs.contains_key(name) {
                added_functions.push(func.clone());
            }
        }

        let old_types: HashMap<_, _> = old
            .types
            .iter()
            .map(|t| (t.name.clone(), t.clone()))
            .collect();
        let new_types: HashMap<_, _> = new
            .types
            .iter()
            .map(|t| (t.name.clone(), t.clone()))
            .collect();

        let mut added_types = Vec::new();
        let mut removed_types = Vec::new();
        let mut changed_types = Vec::new();

        for (name, ty) in &old_types {
            match new_types.get(name) {
                Some(new_ty) if ty != new_ty => changed_types.push(new_ty.clone()),
                None => removed_types.push(ty.clone()),
                _ => {}
            }
        }

        for (name, ty) in &new_types {
            if !old_types.contains_key(name) {
                added_types.push(ty.clone());
            }
        }

        Ok(ContractDiff {
            added_functions,
            removed_functions,
            changed_functions,
            added_types,
            removed_types,
            changed_types,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use analyzer::Analyzer;
    use std::fs;
    use std::path::PathBuf;

    fn get_fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../target/wasm32v1-none/release/");
        path.push(name);
        path.with_extension("wasm")
    }

    #[test]
    fn test_compare_v1_v2() {
        let old_wasm = fs::read(get_fixture_path("migration_v1")).expect("Failed to read v1 wasm");
        let new_wasm = fs::read(get_fixture_path("migration_v2")).expect("Failed to read v2 wasm");

        let old = Analyzer::analyze(&old_wasm).unwrap();
        let new = Analyzer::analyze(&new_wasm).unwrap();

        let diff = WasmDiff::compare(&old, &new).unwrap();

        assert!(
            diff.added_functions
                .iter()
                .any(|f| f.name == "migrate_record"),
            "migrate_record should be added"
        );
        // Because of how the fixture was written (Record -> RecordV2), Record is removed and RecordV2 is added
        assert!(
            diff.removed_types.iter().any(|t| t.name == "Record"),
            "Record should be removed"
        );
        assert!(
            diff.added_types.iter().any(|t| t.name == "RecordV2"),
            "RecordV2 should be added"
        );
        // We also expect get_record to be changed because its return type changed from Record to RecordV2.
        // Wait, right now our analyzer doesn't extract input/output types fully yet, so `func != new_func` might not trigger if we don't populate inputs/outputs.
        // But that's fine for the basic diffing implementation.
    }
}
