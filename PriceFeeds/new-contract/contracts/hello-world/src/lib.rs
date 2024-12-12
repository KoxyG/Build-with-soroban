#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

mod reflector_contract {
    soroban_sdk::contractimport!(
        file = "../../../reflector-contract/target/wasm32-unknown-unknown/release/reflector_oracle.wasm"
    );
}


#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn add_with(env: Env, contract: Address, x: u32, y: u32) -> u32 {
        let client = reflector_contract::Client::new(&env, &contract);
        
    }
}

mod test;
