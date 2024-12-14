#![cfg(test)]
use super::*;
use soroban_sdk::{Env, testutils::EnvTestConfig};
use soroban_sdk::{testutils::{Address as _}, String, Address};


/// Test Setup Information
/// ---------------------
/// Oracle Address: CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN
/// Tests contract initialization and configuration storage

#[test]
fn test_init() {
    // Setup environment with snapshot
    let mut env: Env = Env::from_ledger_snapshot_file("../../snapshot.json");
    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    // Register the contract
    let contract_id = env.register_contract(None, LendingProtocol);
    let client = LendingProtocolClient::new(&env, &contract_id);

    // Create address for the reflector contract
    let reflector_address = Address::from_string(
        &soroban_sdk::String::from_slice(
            &env,
            "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN"
        )
    );

    // Generate admin address
    let admin = Address::generate(&env);

    // Create configuration
    let config = Config {
        oracle_address: reflector_address,
        admin,
        min_loan: 100_0000000,
        max_loan: 10000_0000000,
    };

    // Initialize the contract
    client.initialize(&config);

    // Verify stored configuration
    let stored_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
    assert_eq!(stored_oracle, config.oracle_address);
}
