#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Record(Address),
}

#[contracttype]
pub struct Record {
    pub owner: Address,
    pub value: u64,
}

#[contract]
pub struct MigrationV1Contract;

#[contractimpl]
impl MigrationV1Contract {
    pub fn create_record(env: Env, owner: Address, value: u64) {
        owner.require_auth();
        let key = DataKey::Record(owner.clone());
        if env.storage().persistent().has(&key) {
            panic!("record already exists");
        }
        let record = Record { owner: owner.clone(), value };
        env.storage().persistent().set(&key, &record);
    }

    pub fn get_record(env: Env, owner: Address) -> Option<Record> {
        let key = DataKey::Record(owner);
        env.storage().persistent().get(&key)
    }

    pub fn update_record(env: Env, owner: Address, value: u64) {
        owner.require_auth();
        let key = DataKey::Record(owner.clone());
        if let Some(mut record) = env.storage().persistent().get::<_, Record>(&key) {
            record.value = value;
            env.storage().persistent().set(&key, &record);
        } else {
            panic!("record does not exist");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_create_and_get() {
        let env = Env::default();
        let contract_id = env.register(MigrationV1Contract, ());
        let client = MigrationV1ContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        env.mock_all_auths();
        
        client.create_record(&owner, &100);
        let record = client.get_record(&owner).unwrap();
        assert_eq!(record.owner, owner);
        assert_eq!(record.value, 100);
    }

    #[test]
    fn test_update() {
        let env = Env::default();
        let contract_id = env.register(MigrationV1Contract, ());
        let client = MigrationV1ContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        env.mock_all_auths();
        
        client.create_record(&owner, &100);
        client.update_record(&owner, &200);
        
        let record = client.get_record(&owner).unwrap();
        assert_eq!(record.value, 200);
    }
}
