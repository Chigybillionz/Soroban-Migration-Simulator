#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Record(Address),
}

#[contracttype]
pub struct RecordV1 {
    pub owner: Address,
    pub value: u64,
}

#[contracttype]
pub struct RecordV2 {
    pub owner: Address,
    pub value: u64,
    pub version: u32,
}

#[contract]
pub struct MigrationV2Contract;

#[contractimpl]
impl MigrationV2Contract {
    pub fn create_record(env: Env, owner: Address, value: u64) {
        owner.require_auth();
        let key = DataKey::Record(owner.clone());
        if env.storage().persistent().has(&key) {
            panic!("record already exists");
        }
        let record = RecordV2 { owner: owner.clone(), value, version: 2 };
        env.storage().persistent().set(&key, &record);
    }

    pub fn get_record(env: Env, owner: Address) -> Option<RecordV2> {
        let key = DataKey::Record(owner);
        env.storage().persistent().get(&key)
    }

    pub fn update_record(env: Env, owner: Address, value: u64) {
        owner.require_auth();
        let key = DataKey::Record(owner.clone());
        if let Some(mut record) = env.storage().persistent().get::<_, RecordV2>(&key) {
            record.value = value;
            env.storage().persistent().set(&key, &record);
        } else {
            panic!("record does not exist");
        }
    }

    pub fn migrate_record(env: Env, owner: Address) {
        let key = DataKey::Record(owner.clone());
        
        if let Some(old_record) = env.storage().persistent().get::<_, RecordV1>(&key) {
            let new_record = RecordV2 {
                owner: old_record.owner,
                value: old_record.value,
                version: 2,
            };
            env.storage().persistent().set(&key, &new_record);
        } else {
            panic!("v1 record not found");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_create_v2() {
        let env = Env::default();
        let contract_id = env.register(MigrationV2Contract, ());
        let client = MigrationV2ContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        env.mock_all_auths();
        
        client.create_record(&owner, &100);
        let record = client.get_record(&owner).unwrap();
        assert_eq!(record.owner, owner);
        assert_eq!(record.value, 100);
        assert_eq!(record.version, 2);
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        let env = Env::default();
        let contract_id = env.register(MigrationV2Contract, ());
        let client = MigrationV2ContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        
        // Test 1: Create V1 state directly in storage (bypassing auth/API since it's prior state)
        let key = DataKey::Record(owner.clone());
        let old_record = RecordV1 { owner: owner.clone(), value: 500 };
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&key, &old_record);
        });

        // Test 2: Migrate
        client.migrate_record(&owner);

        // Test 3: Invariants preserved
        let new_record = client.get_record(&owner).unwrap();
        assert_eq!(new_record.owner, old_record.owner, "Invariant failed: owner changed");
        assert_eq!(new_record.value, old_record.value, "Invariant failed: value changed");
        assert_eq!(new_record.version, 2, "Migration failed: version not initialized to 2");
    }
}
