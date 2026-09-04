use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Malformed JSON: {0}")]
    MalformedJson(#[from] serde_json::Error),
    #[error("Duplicate state key detected for durability {0:?} and key {1:?}")]
    DuplicateKey(Durability, StateValue),
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "type", content = "value")]
pub enum StateValue {
    Address(String),
    U32(u32),
    U64(u64),
    Bool(bool),
    Symbol(String),
    Bytes(String),
    Struct(BTreeMap<String, StateValue>),
    Map(Vec<(StateValue, StateValue)>),
    Vec(Vec<StateValue>),
    Option(Option<Box<StateValue>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Durability {
    Persistent,
    Instance,
    Temporary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateEntry {
    pub durability: Durability,
    pub key: StateValue,
    pub value: StateValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ContractState {
    pub contract_id: String,
    pub entries: Vec<StateEntry>,
}

impl ContractState {
    pub fn new(contract_id: String, entries: Vec<StateEntry>) -> Result<Self, StateError> {
        let mut state = Self {
            contract_id,
            entries,
        };
        state.validate()?;
        state.canonicalize();
        Ok(state)
    }

    pub fn from_json(json: &str) -> Result<Self, StateError> {
        let mut state: Self = serde_json::from_str(json)?;
        state.validate()?;
        state.canonicalize();
        Ok(state)
    }

    pub fn to_json(&self) -> Result<String, StateError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_file(path: &str) -> Result<Self, StateError> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    pub fn validate(&self) -> Result<(), StateError> {
        let mut seen = HashSet::new();
        for entry in &self.entries {
            let key_tuple = (entry.durability.clone(), entry.key.clone());
            if !seen.insert(key_tuple.clone()) {
                return Err(StateError::DuplicateKey(key_tuple.0, key_tuple.1));
            }
        }
        Ok(())
    }

    pub fn canonicalize(&mut self) {
        self.entries.sort_unstable_by(|a, b| {
            a.durability
                .cmp(&b.durability)
                .then_with(|| a.key.cmp(&b.key))
        });
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        let canonical_json = serde_json::to_string(self).unwrap();
        hasher.update(canonical_json.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_v1_json() -> String {
        r#"{
            "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
            "entries": [
                {
                    "durability": "Persistent",
                    "key": { "type": "Address", "value": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF" },
                    "value": {
                        "type": "Struct",
                        "value": {
                            "owner": { "type": "Address", "value": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF" },
                            "value": { "type": "U64", "value": 100 }
                        }
                    }
                }
            ]
        }"#.to_string()
    }

    fn dummy_v2_json() -> String {
        r#"{
            "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
            "entries": [
                {
                    "durability": "Persistent",
                    "key": { "type": "Address", "value": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF" },
                    "value": {
                        "type": "Struct",
                        "value": {
                            "owner": { "type": "Address", "value": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF" },
                            "value": { "type": "U64", "value": 100 },
                            "version": { "type": "U32", "value": 2 }
                        }
                    }
                }
            ]
        }"#.to_string()
    }

    #[test]
    fn test_a_deserialize_v1_state() {
        let state = ContractState::from_json(&dummy_v1_json()).unwrap();
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].durability, Durability::Persistent);
    }

    #[test]
    fn test_b_deserialize_v2_state() {
        let state = ContractState::from_json(&dummy_v2_json()).unwrap();
        assert_eq!(state.entries.len(), 1);

        if let StateValue::Struct(fields) = &state.entries[0].value {
            assert!(fields.contains_key("version"));
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_c_canonical_ordering() {
        let entry1 = StateEntry {
            durability: Durability::Persistent,
            key: StateValue::U32(1),
            value: StateValue::U64(100),
        };
        let entry2 = StateEntry {
            durability: Durability::Instance,
            key: StateValue::U32(2),
            value: StateValue::U64(200),
        };

        let mut state_a =
            ContractState::new("C1".to_string(), vec![entry1.clone(), entry2.clone()]).unwrap();
        let mut state_b = ContractState::new("C1".to_string(), vec![entry2, entry1]).unwrap();

        state_a.canonicalize();
        state_b.canonicalize();

        assert_eq!(state_a.entries, state_b.entries);
    }

    #[test]
    fn test_d_fingerprint_determinism() {
        let entry1 = StateEntry {
            durability: Durability::Persistent,
            key: StateValue::U32(1),
            value: StateValue::U64(100),
        };
        let entry2 = StateEntry {
            durability: Durability::Instance,
            key: StateValue::U32(2),
            value: StateValue::U64(200),
        };

        let state_a =
            ContractState::new("C1".to_string(), vec![entry1.clone(), entry2.clone()]).unwrap();
        let state_b = ContractState::new("C1".to_string(), vec![entry2, entry1]).unwrap();

        assert_eq!(state_a.fingerprint(), state_b.fingerprint());
    }

    #[test]
    fn test_e_state_mutation() {
        let mut state = ContractState::from_json(&dummy_v1_json()).unwrap();
        let old_fingerprint = state.fingerprint();

        state.entries[0].value = StateValue::U64(999); // mutate
        state.canonicalize();

        let new_fingerprint = state.fingerprint();
        assert_ne!(old_fingerprint, new_fingerprint);
    }

    #[test]
    fn test_f_duplicate_key_detection() {
        let json = r#"{
            "contract_id": "C1",
            "entries": [
                {
                    "durability": "Persistent",
                    "key": { "type": "U32", "value": 1 },
                    "value": { "type": "U64", "value": 100 }
                },
                {
                    "durability": "Persistent",
                    "key": { "type": "U32", "value": 1 },
                    "value": { "type": "U64", "value": 200 }
                }
            ]
        }"#;

        let res = ContractState::from_json(json);
        assert!(matches!(res, Err(StateError::DuplicateKey(_, _))));
    }

    #[test]
    fn test_g_invalid_fixture() {
        let res = ContractState::from_json("{ invalid json ]");
        assert!(matches!(res, Err(StateError::MalformedJson(_))));
    }

    #[test]
    fn test_h_serialization_round_trip() {
        let state = ContractState::from_json(&dummy_v1_json()).unwrap();
        let json = state.to_json().unwrap();
        let state2 = ContractState::from_json(&json).unwrap();

        assert_eq!(state, state2);
        assert_eq!(state.fingerprint(), state2.fingerprint());
    }
}
