use serde::{Deserialize, Serialize};
use stellar_xdr::curr::{Limits, Limited, ReadXdr, ScEnvMetaEntry, ScMetaEntry, ScSpecEntry, ScSpecTypeDef};
use std::collections::HashMap;
use thiserror::Error;
use wasmparser::{Parser, Payload};

#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("Invalid WASM: {0}")]
    InvalidWasm(String),
    #[error("Missing contractspecv0 section")]
    MissingContractSpec,
    #[error("XDR Decode Error: {0}")]
    XdrDecodeError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContractAnalysis {
    pub has_metadata: bool,
    pub has_env_metadata: bool,
    pub env_metadata: Option<EnvMetadata>,
    pub metadata: Option<HashMap<String, String>>,
    pub functions: Vec<FunctionAnalysis>,
    pub types: Vec<TypeAnalysis>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EnvMetadata {
    pub interface_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionAnalysis {
    pub name: String,
    pub inputs: Vec<FieldAnalysis>,
    pub outputs: Vec<TypeRef>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TypeAnalysis {
    pub name: String,
    pub description: String,
    pub fields: Vec<FieldAnalysis>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldAnalysis {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TypeRef {
    Simple(String),
    Complex(String),
    Unknown,
}

impl TypeRef {
    pub fn to_string_rep(&self) -> String {
        match self {
            TypeRef::Simple(s) => s.clone(),
            TypeRef::Complex(s) => s.clone(),
            TypeRef::Unknown => "Unknown".to_string(),
        }
    }
}

pub struct Analyzer;

impl Analyzer {
    #[allow(clippy::collapsible_if)]
    pub fn analyze(wasm_bytes: &[u8]) -> Result<ContractAnalysis, AnalyzerError> {
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut found_spec = false;
        
        let mut has_metadata = false;
        let mut has_env_metadata = false;
        let mut metadata = HashMap::new();
        let mut env_metadata = None;

        for payload in Parser::new(0).parse_all(wasm_bytes) {
            let payload = payload.map_err(|e| AnalyzerError::InvalidWasm(e.to_string()))?;
            if let Payload::CustomSection(section) = payload {
                let name = section.name();
                if name == "contractspecv0" {
                    found_spec = true;
                    let data = section.data();
                    let mut cursor = std::io::Cursor::new(data);
                    let limits = Limits::none();
                    let mut limited = Limited::new(&mut cursor, limits);
                    
                    while (limited.inner.position() as usize) < data.len() {
                        let entry = ScSpecEntry::read_xdr(&mut limited)
                            .map_err(|e| AnalyzerError::XdrDecodeError(e.to_string()))?;
                        
                        match entry {
                            ScSpecEntry::FunctionV0(f) => {
                                let name = f.name.to_utf8_string_lossy().to_string();
                                let mut inputs = Vec::new();
                                for input in f.inputs.iter() {
                                    inputs.push(FieldAnalysis {
                                        name: input.name.to_utf8_string_lossy().to_string(),
                                        type_ref: map_type_def(&input.type_),
                                    });
                                }
                                let mut outputs = Vec::new();
                                for output in f.outputs.iter() {
                                    outputs.push(map_type_def(output));
                                }
                                functions.push(FunctionAnalysis { name, inputs, outputs });
                            }
                            ScSpecEntry::UdtStructV0(s) => {
                                let name = s.name.to_utf8_string_lossy().to_string();
                                let mut fields = Vec::new();
                                for field in s.fields.iter() {
                                    fields.push(FieldAnalysis {
                                        name: field.name.to_utf8_string_lossy().to_string(),
                                        type_ref: map_type_def(&field.type_),
                                    });
                                }
                                types.push(TypeAnalysis { name, description: "struct".to_string(), fields });
                            }
                            ScSpecEntry::UdtUnionV0(u) => {
                                let name = u.name.to_utf8_string_lossy().to_string();
                                let mut fields = Vec::new();
                                for case in u.cases.iter() {
                                    match case {
                                        stellar_xdr::curr::ScSpecUdtUnionCaseV0::VoidV0(c) => {
                                            fields.push(FieldAnalysis {
                                                name: c.name.to_utf8_string_lossy().to_string(),
                                                type_ref: TypeRef::Simple("void".to_string()),
                                            });
                                        }
                                        stellar_xdr::curr::ScSpecUdtUnionCaseV0::TupleV0(c) => {
                                            let mut type_strs = Vec::new();
                                            for ty in c.type_.iter() {
                                                type_strs.push(map_type_def(ty).to_string_rep());
                                            }
                                            fields.push(FieldAnalysis {
                                                name: c.name.to_utf8_string_lossy().to_string(),
                                                type_ref: TypeRef::Complex(format!("Tuple<{}>", type_strs.join(", "))),
                                            });
                                        }
                                    }
                                }
                                types.push(TypeAnalysis { name, description: "union".to_string(), fields });
                            }
                            ScSpecEntry::UdtEnumV0(e) => {
                                let name = e.name.to_utf8_string_lossy().to_string();
                                let mut fields = Vec::new();
                                for case in e.cases.iter() {
                                    fields.push(FieldAnalysis {
                                        name: case.name.to_utf8_string_lossy().to_string(),
                                        type_ref: TypeRef::Simple(case.value.to_string()),
                                    });
                                }
                                types.push(TypeAnalysis { name, description: "enum".to_string(), fields });
                            }
                            _ => {}
                        }
                    }
                } else if name == "contractmetav0" {
                    has_metadata = true;
                    let data = section.data();
                    let mut cursor = std::io::Cursor::new(data);
                    let limits = Limits::none();
                    let mut limited = Limited::new(&mut cursor, limits);
                    while (limited.inner.position() as usize) < data.len() {
                        if let Ok(ScMetaEntry::ScMetaV0(m)) = ScMetaEntry::read_xdr(&mut limited) {
                            metadata.insert(m.key.to_utf8_string_lossy().to_string(), m.val.to_utf8_string_lossy().to_string());
                        } else {
                            break;
                        }
                    }
                } else if name == "contractenvmetav0" {
                    has_env_metadata = true;
                    let data = section.data();
                    let mut cursor = std::io::Cursor::new(data);
                    let limits = Limits::none();
                    let mut limited = Limited::new(&mut cursor, limits);
                    while (limited.inner.position() as usize) < data.len() {
                        if let Ok(stellar_xdr::curr::ScEnvMetaEntry::ScEnvMetaKindInterfaceVersion(v)) = ScEnvMetaEntry::read_xdr(&mut limited) {
                            env_metadata = Some(EnvMetadata { interface_version: v.protocol });
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if !found_spec {
            return Err(AnalyzerError::MissingContractSpec);
        }

        Ok(ContractAnalysis {
            has_metadata,
            has_env_metadata,
            env_metadata,
            metadata: if metadata.is_empty() { None } else { Some(metadata) },
            functions,
            types,
        })
    }
}

fn map_type_def(type_def: &ScSpecTypeDef) -> TypeRef {
    match type_def {
        ScSpecTypeDef::Val => TypeRef::Simple("Val".to_string()),
        ScSpecTypeDef::U64 => TypeRef::Simple("u64".to_string()),
        ScSpecTypeDef::I64 => TypeRef::Simple("i64".to_string()),
        ScSpecTypeDef::U32 => TypeRef::Simple("u32".to_string()),
        ScSpecTypeDef::I32 => TypeRef::Simple("i32".to_string()),
        ScSpecTypeDef::U128 => TypeRef::Simple("u128".to_string()),
        ScSpecTypeDef::I128 => TypeRef::Simple("i128".to_string()),
        ScSpecTypeDef::U256 => TypeRef::Simple("u256".to_string()),
        ScSpecTypeDef::I256 => TypeRef::Simple("i256".to_string()),
        ScSpecTypeDef::Bool => TypeRef::Simple("bool".to_string()),
        ScSpecTypeDef::Symbol => TypeRef::Simple("symbol".to_string()),
        ScSpecTypeDef::String => TypeRef::Simple("string".to_string()),
        ScSpecTypeDef::Bytes => TypeRef::Simple("bytes".to_string()),
        ScSpecTypeDef::Address => TypeRef::Simple("Address".to_string()),
        ScSpecTypeDef::Option(o) => TypeRef::Complex(format!("Option<{}>", map_type_def(&o.value_type).to_string_rep())),
        ScSpecTypeDef::Result(r) => TypeRef::Complex(format!("Result<{}, {}>", map_type_def(&r.ok_type).to_string_rep(), map_type_def(&r.error_type).to_string_rep())),
        ScSpecTypeDef::Vec(v) => TypeRef::Complex(format!("Vec<{}>", map_type_def(&v.element_type).to_string_rep())),
        ScSpecTypeDef::Map(m) => TypeRef::Complex(format!("Map<{}, {}>", map_type_def(&m.key_type).to_string_rep(), map_type_def(&m.value_type).to_string_rep())),
        ScSpecTypeDef::Tuple(t) => {
            let mut type_strs = Vec::new();
            for ty in t.value_types.iter() {
                type_strs.push(map_type_def(ty).to_string_rep());
            }
            TypeRef::Complex(format!("Tuple<{}>", type_strs.join(", ")))
        },
        ScSpecTypeDef::BytesN(b) => TypeRef::Complex(format!("BytesN<{}>", b.n)),
        ScSpecTypeDef::Udt(u) => TypeRef::Complex(u.name.to_utf8_string_lossy().to_string()),
        ScSpecTypeDef::Void => TypeRef::Simple("void".to_string()),
        ScSpecTypeDef::Timepoint => TypeRef::Simple("Timepoint".to_string()),
        ScSpecTypeDef::Duration => TypeRef::Simple("Duration".to_string()),
        _ => TypeRef::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../target/wasm32v1-none/release/");
        path.push(name);
        path.with_extension("wasm")
    }

    #[test]
    fn test_analyze_v1() {
        let wasm = fs::read(get_fixture_path("migration_v1")).expect("Failed to read v1 wasm");
        let analysis = Analyzer::analyze(&wasm).unwrap();
        
        let has_create = analysis.functions.iter().any(|f| f.name == "create_record");
        assert!(has_create, "Missing create_record function");
    }

    #[test]
    fn test_analyze_v2() {
        let wasm = fs::read(get_fixture_path("migration_v2")).expect("Failed to read v2 wasm");
        let analysis = Analyzer::analyze(&wasm).unwrap();
        
        let has_migrate = analysis.functions.iter().any(|f| f.name == "migrate_record");
        assert!(has_migrate, "Missing migrate_record function");
    }

    #[test]
    fn test_invalid_wasm() {
        let res = Analyzer::analyze(&[0, 1, 2, 3]);
        assert!(matches!(res, Err(AnalyzerError::InvalidWasm(_))));
    }

    #[test]
    fn test_missing_spec() {
        let wasm = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let res = Analyzer::analyze(&wasm);
        assert!(matches!(res, Err(AnalyzerError::MissingContractSpec)));
    }

    #[test]
    fn test_without_metadata() {
        let wasm = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x00,
            0x0f,
            0x0e,
            b'c', b'o', b'n', b't', b'r', b'a', b'c', b't', b's', b'p', b'e', b'c', b'v', b'0'
        ];
        let res = Analyzer::analyze(&wasm).unwrap();
        assert!(!res.has_metadata);
        assert!(!res.has_env_metadata);
        assert!(res.metadata.is_none());
    }

    #[test]
    fn test_metadata_extraction() {
        let wasm = fs::read(get_fixture_path("migration_v1")).unwrap();
        let analysis = Analyzer::analyze(&wasm).unwrap();
        
        assert!(analysis.has_metadata);
        assert!(analysis.has_env_metadata);
        assert!(analysis.metadata.is_some());
        assert!(analysis.env_metadata.is_some());
    }

    #[test]
    fn test_record_fields_v1() {
        let wasm = fs::read(get_fixture_path("migration_v1")).unwrap();
        let analysis = Analyzer::analyze(&wasm).unwrap();
        let record = analysis.types.iter().find(|t| t.name == "Record").unwrap();
        
        assert_eq!(record.fields.len(), 2);
        assert_eq!(record.fields[0].name, "owner");
        assert_eq!(record.fields[1].name, "value");
    }

    #[test]
    fn test_record_fields_v2() {
        let wasm = fs::read(get_fixture_path("migration_v2")).unwrap();
        let analysis = Analyzer::analyze(&wasm).unwrap();
        let record_v2 = analysis.types.iter().find(|t| t.name == "RecordV2").unwrap();
        
        assert_eq!(record_v2.fields.len(), 3);
        assert_eq!(record_v2.fields[0].name, "owner");
        assert_eq!(record_v2.fields[1].name, "value");
        assert_eq!(record_v2.fields[2].name, "version");
    }
}
