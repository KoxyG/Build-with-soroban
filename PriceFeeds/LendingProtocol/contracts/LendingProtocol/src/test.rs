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
    let contract_id = env.register_contract(None, LendingProtocol);
    let client = LendingProtocolClient::new(&env, &contract_id);

    // Create address for the reflector contract
    let reflector_address = Address::from_string(
        &String::from_slice(
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

    // Verify stored configuration - Wrap in as_contract
    env.as_contract(&contract_id, || {
        let stored_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
        assert_eq!(stored_oracle, config.oracle_address);
    });
}


// #[test]
// fn test_loan_repayment() {
//     // Setup environment with snapshot
//     let mut env: Env = Env::from_ledger_snapshot_file("../../snapshot.json");
    
//     env.set_config(EnvTestConfig {
//         capture_snapshot_at_drop: false,
//     });

//     // Register the lending contract
//     let contract_id = env.register_contract(None, LendingProtocol);
//     let client = LendingProtocolClient::new(&env, &contract_id);

//     // Use the same oracle address as in your working test
//     let oracle_address = Address::from_string(
//         &String::from_str(
//             &env,
//             "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN"
//         )
//     );

//     // Initialize the contract first
//     let admin = Address::generate(&env);
//     let config = Config {
//         oracle_address: oracle_address.clone(),
//         admin: admin.clone(),
//         min_loan: 100_0000000,
//         max_loan: 10000_0000000,
//     };
//     client.initialize(&config);

//     // Test variables
//     let borrower = Address::generate(&env);
//     let token = Address::generate(&env);
    
//     // Create loan parameters
//     let amount = 1000_0000000; // 1000 tokens
//     let interest_rate = 5; // 5%
//     let duration = 30 * DAY_IN_LEDGERS; // 30 days
    
//     // Create asset info
//     let collateral_asset = AssetInfo {
//         code: String::from_str(&env, "XLM"),
//         issuer: Address::generate(&env),
//     };
//     let collateral_amount = 2000_0000000; // 2000 tokens as collateral

//     // Create the loan
//     let loan_id = client.create_loan(
//         &amount,
//         &token,
//         &interest_rate,
//         &duration,
//         &borrower,
//         &collateral_asset,
//         &collateral_amount
//     );

//     // Verify loan
//     let loan = client.get_loan(&loan_id).expect("Loan should exist");
//     assert_eq!(loan.amount, amount);
//     assert_eq!(loan.borrower, borrower);
//     assert_eq!(loan.token, token);
//     assert_eq!(loan.interest_rate, interest_rate);
//     assert_eq!(loan.duration, duration);
//     assert_eq!(loan.collateral_asset, collateral_asset);
//     assert_eq!(loan.collateral_amount, collateral_amount);
//     assert!(loan.active);
//     assert!(!loan.repaid);
// }