use soroban_sdk::{
    xdr::{
        ContractDataDurability, ContractDataEntry, ContractId, Hash, LedgerEntryData, ScAddress,
        ScVal,
    },
    Env,
};

use state_engine::{ContractState, Durability, StateEntry};

use crate::conversion::scval_to_state_value;
use crate::MigrationError;

/// Map XDR ContractDataDurability to state_engine Durability.
fn map_durability(d: ContractDataDurability) -> Durability {
    match d {
        ContractDataDurability::Persistent => Durability::Persistent,
        ContractDataDurability::Temporary => Durability::Temporary,
    }
}

/// Extract the 32-byte contract ID from an ScAddress, if it is a contract.
fn contract_id_bytes_from_scaddress(addr: &ScAddress) -> Option<[u8; 32]> {
    match addr {
        ScAddress::Contract(ContractId(Hash(bytes))) => Some(*bytes),
        _ => None,
    }
}

/// Capture contract application state from a LedgerSnapshot.
///
/// Only extracts `LedgerEntryData::ContractData` entries belonging to the
/// specified contract. Filters out:
/// - ContractInstance entries (key = ScVal::LedgerKeyContractInstance)
/// - Temporary storage entries
/// - Entries belonging to other contracts
pub fn capture_state_from_snapshot(
    env: &Env,
    contract_id: &soroban_sdk::Address,
    snapshot: &soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<ContractState, MigrationError> {
    let mut entries = Vec::new();

    // Get the contract's XDR identifier for filtering.
    // Convert SDK Address → ScAddress → extract 32-byte contract ID.
    let sc_address: ScAddress = contract_id.into();
    let contract_bytes = match sc_address {
        ScAddress::Contract(ContractId(Hash(bytes))) => bytes,
        _ => {
            return Err(MigrationError::StateCaptureFailure(
                "Contract ID is not a contract address".to_string(),
            ))
        }
    };

    for (_ledger_key, (entry_xdr, _live_until)) in snapshot.entries() {
        if let LedgerEntryData::ContractData(ContractDataEntry {
            contract,
            key,
            val,
            durability,
            ..
        }) = &entry_xdr.data
        {
            // Filter: only Persistent entries (skip Temporary)
            if *durability == ContractDataDurability::Temporary {
                continue;
            }

            // Filter: skip ContractInstance entry
            if matches!(key, ScVal::LedgerKeyContractInstance) {
                continue;
            }

            // Filter: only entries belonging to this contract
            if let Some(entry_contract_bytes) = contract_id_bytes_from_scaddress(contract) {
                if entry_contract_bytes != contract_bytes {
                    continue;
                }
            } else {
                // Not a contract address (e.g. Account), skip
                continue;
            }

            // Convert key ScVal → StateValue
            let state_key = scval_to_state_value(env, key).map_err(|e| {
                MigrationError::StateConversionFailure(format!(
                    "Failed to convert key ScVal: {:?}",
                    e
                ))
            })?;

            // Convert value ScVal → StateValue
            let state_val = scval_to_state_value(env, val).map_err(|e| {
                MigrationError::StateConversionFailure(format!(
                    "Failed to convert value ScVal: {:?}",
                    e
                ))
            })?;

            entries.push(StateEntry {
                durability: map_durability(*durability),
                key: state_key,
                value: state_val,
            });
        }
    }

    // Build ContractState (validate + canonicalize)
    ContractState::new(format!("{}", contract_id.to_string()), entries)
        .map_err(|e| MigrationError::StateCaptureFailure(format!("{}", e)))
}
