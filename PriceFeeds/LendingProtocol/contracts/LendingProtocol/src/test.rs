#![cfg(test)]
use super::*;
use soroban_sdk::{Env, testutils::EnvTestConfig};
use soroban_sdk::{testutils::{Address as _}, String, Address};

#[test]
fn test_init() {
    // Setup environment with snapshot
    let mut env: Env = Env::from_ledger_snapshot_file("../../snapshot.json");
    
    // Set config before any operations
    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    // Register the contract
    
    let contract_id = env.register(LendingProtocol, ());
    let client = LendingProtocolClient::new(&env, &contract_id);

    // MAINNET
    // Create address for the reflector contract
    // let reflector_address = Address::from_string(
    //     &String::from_slice(
    //         &env,
    //         "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN"
    //     )
    // );

    // TESTNET
    // Create address for the reflector contract
    let reflector_address = Address::from_string(
        &String::from_slice(
            &env,
            "CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP"
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

    // Verify stored configuration - Wrap in as_contract
    env.as_contract(&contract_id, || {
        let stored_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
        assert_eq!(stored_oracle, config.oracle_address);
    });
}

// #[test]
// fn test_loan_repayment() {
//     let mut env: Env = Env::default();
//     let contract_id = env.register(LendingProtocol, ());
//     let client = LendingProtocolClient::new(&env, &contract_id);

//     // Initialize with proper minimum values
//     let config = Config {
//         oracle_address: oracle_address.clone(),
//         admin: admin.clone(),
//         min_loan: 100_0000000,
//         max_loan: 10000_0000000,
//     };
//     client.initialize(&config);

//     // Use proper amounts
//     let amount = 1000_0000000; // Meets minimum requirement
//     let collateral_amount = 2000_0000000;

//     // Create loan with proper values
//     let result = client.create_loan(
//         &amount,
//         &token,
//         &interest_rate,
//         &duration,
//         &borrower,
//         &collateral_ass                                                                                          ,
//         &collateral_amount
//     );

  
   
// }
