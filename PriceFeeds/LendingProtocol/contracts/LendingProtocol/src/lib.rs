#![no_std]
use soroban_sdk::{
    symbol_short,
    xdr::Asset,
    contract, contractimpl, contracterror, contracttype, Address, Env, Symbol, Vec,
    token::Client as TokenClient,
};

mod reflector_contract {
    soroban_sdk::contractimport!(
        file = "../../../reflector-contract/target/wasm32-unknown-unknown/release/reflector_oracle.wasm"
    );
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Config {
    pub oracle_address: Address,  // Reflector oracle contract address
    pub admin: Address,          // Admin address
    pub min_loan: i128,         // Minimum loan amount
    pub max_loan: i128,         // Maximum loan amount
}



// #[allow(dead_code)]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Loan {
    pub amount: i128,
    pub interest_rate: u32,
    pub duration: u32,
    pub repayment_amount: i128,
    pub funding_deadline: u64,
    pub borrower: Address,
    pub lender: Option<Address>,
    pub collateral_asset: Asset,
    pub token: Address,         // Loan token contract address
    pub collateral_amount: i128,
    pub active: bool,
    pub repaid: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}


#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidInterest = 2,
    InvalidDuration = 3,
    InactiveLoan = 4,
    InsufficientCollateral = 5,
    OracleError = 6,
    Unauthorized = 7,
    DeadlinePassed = 8,
    LoanTooSmall = 9,
    LoanTooLarge = 10,
    InvalidRepaymentAmount = 11,
    TokenTransferFailed = 12,
    CannotLiquidate = 13,
}

const MIN_INTEREST_RATE: u32 = 1;   // 1%
const MAX_INTEREST_RATE: u32 = 10;  // 10%
const DAY_IN_LEDGERS: u32 = 17280;  // 24 hours worth of ledgers

#[contract]
pub struct LendingProtocol;

#[contractimpl]
impl LendingProtocol {
    pub fn __constructor(env: Env, config: Config) {
        // Verify the oracle contract exists and is valid
        let oracle_client = reflector_contract::Client::new(&env, &config.oracle_address);
        let _version = oracle_client.version();

        // Store configuration in contract storage
        env.storage().instance().set(&symbol_short!("oracle"), &config.oracle_address);
        env.storage().instance().set(&symbol_short!("admin"), &config.admin);
        env.storage().instance().set(&symbol_short!("min_loan"), &config.min_loan);
        env.storage().instance().set(&symbol_short!("max_loan"), &config.max_loan);
    }

    pub fn create_loan(
        env: Env,
        amount: i128,
        token: Address,
        interest_rate: u32,
        duration: u32,
        collateral_asset: Asset,
        collateral_amount: i128,
    ) -> Result<u32, Error> {
        let min_loan: i128 = env.storage().instance().get(&symbol_short!("min_loan"))
            .expect("Min loan not set");
        let max_loan: i128 = env.storage().instance().get(&symbol_short!("max_loan"))
            .expect("Max loan not set");

        if amount < min_loan {
            return Err(Error::LoanTooSmall);
        }
        if amount > max_loan {
            return Err(Error::LoanTooLarge);
        }
        if interest_rate < MIN_INTEREST_RATE || interest_rate > MAX_INTEREST_RATE {
            return Err(Error::InvalidInterest);
        }
        if duration == 0 {
            return Err(Error::InvalidDuration);
        }

        // Calculate repayment amount using direct percentage
        let interest_amount = (amount * interest_rate as i128) / 100;
        let repayment_amount = amount + interest_amount;
        
        // Set funding deadline
        let funding_deadline = env.ledger().timestamp() + (DAY_IN_LEDGERS as u64);

        // Verify collateral value
        Self::verify_collateral_value(&env, amount, &collateral_asset, collateral_amount)?;

        let loan = Loan {
            amount,
            interest_rate,
            duration,
            repayment_amount,
            funding_deadline,
            borrower: env.invoker().clone(),
            lender: None,
            collateral_asset,
            token,
            collateral_amount,
            active: true,
            repaid: false,
        };

        let loan_id = Self::get_next_loan_id(&env);
        env.storage().instance().set(&loan_id, &loan);

        Ok(loan_id)
    }

    pub fn fund_loan(
        env: Env, 
        loan_id: u32,
        token: Address,
        amount: i128
    ) -> Result<(), Error> {
        let mut loan: Loan = env.storage().instance().get(&loan_id)
            .ok_or(Error::InactiveLoan)?;

        if !loan.active {
            return Err(Error::InactiveLoan);
        }
        if env.ledger().timestamp() > loan.funding_deadline {
            return Err(Error::DeadlinePassed);
        }
        if amount != loan.amount {
            return Err(Error::InvalidAmount);
        }

        // Create token client
        let token_client = TokenClient::new(&env, &token);

        // Transfer tokens from lender to borrower
        token_client.transfer(
            &env.invoker(),
            &loan.borrower,
            &amount
        );

        loan.lender = Some(env.invoker().clone());
        loan.active = false;

        env.storage().instance().set(&loan_id, &loan);
        Ok(())
    }

    pub fn repay_loan(
        env: Env, 
        loan_id: u32,
        token: Address,
        amount: i128
    ) -> Result<(), Error> {
        let mut loan: Loan = env.storage().instance().get(&loan_id)
            .ok_or(Error::InactiveLoan)?;

        if env.invoker() != loan.borrower {
            return Err(Error::Unauthorized);
        }
        if amount != loan.repayment_amount {
            return Err(Error::InvalidRepaymentAmount);
        }

        let lender = loan.lender.clone().ok_or(Error::InactiveLoan)?;

        // Create token client
        let token_client = TokenClient::new(&env, &token);

        // Transfer repayment amount from borrower to lender
        token_client.transfer(
            &loan.borrower,
            &lender,
            &amount
        );

        loan.repaid = true;
        loan.active = false;

        env.storage().instance().set(&loan_id, &loan);
        Ok(())
    }

    pub fn get_loan(env: Env, loan_id: u32) -> Option<Loan> {
        env.storage().instance().get(&loan_id)
    }

    pub fn get_active_loans(env: Env) -> Vec<(u32, Loan)> {
        let mut active_loans = Vec::new(&env);
        let loan_count = Self::get_next_loan_id(&env);

        for id in 0..loan_count {
            if let Some(loan) = env.storage().instance().get(&id) {
                if loan.active {
                    active_loans.push_back((id, loan));
                }
            }
        }
        active_loans
    }

    pub fn get_cross_asset_price(
        env: Env,
        base_asset: Asset,
        quote_asset: Asset,
    ) -> Result<PriceData, Error> {
        let oracle_address: Address = env.storage().instance().get(&symbol_short!("oracle"))
            .ok_or(Error::OracleError)?;
        
        let oracle = reflector_contract::Client::new(&env, &oracle_address);
        
        let price_data = oracle.x_last_price(&base_asset, &quote_asset)
            .ok_or(Error::OracleError)?;
            
        Ok(price_data)
    }

    pub fn liquidate(env: Env, loan_id: u32) -> Result<(), Error> {
        let loan: Loan = env.storage().instance().get(&loan_id)
            .ok_or(Error::InactiveLoan)?;

        if loan.repaid {
            return Err(Error::InactiveLoan);
        }

        let oracle_address: Address = env.storage().instance()
            .get(&symbol_short!("oracle"))
            .ok_or(Error::OracleError)?;
        
        let oracle = reflector_contract::Client::new(&env, &oracle_address);
        let price_data = oracle.lastprice(&loan.collateral_asset)
            .ok_or(Error::OracleError)?;

        let decimals = oracle.decimals();
        let collateral_value = (loan.collateral_amount * price_data.price) 
            / 10i128.pow(decimals);
        
        // Check if below 120% collateralization
        if collateral_value >= (loan.amount * 120) / 100 {
            return Err(Error::CannotLiquidate);
        }

        // Handle liquidation logic here
        // Transfer collateral to liquidator
        // Transfer remaining value to borrower

        env.storage().instance().set(&loan_id, &Loan {
            active: false,
            repaid: true,
            ..loan
        });

        Ok(())
    }

    pub fn update_oracle(env: Env, new_oracle: Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance()
            .get(&symbol_short!("admin"))
            .ok_or(Error::Unauthorized)?;

        if env.invoker() != admin {
            return Err(Error::Unauthorized);
        }

        let oracle_client = reflector_contract::Client::new(&env, &new_oracle);
        let _version = oracle_client.version();

        env.storage().instance().set(&symbol_short!("oracle"), &new_oracle);
        Ok(())
    }

    fn verify_collateral_value(
        env: &Env,
        loan_amount: i128,
        collateral_asset: &Asset,
        collateral_amount: i128,
    ) -> Result<(), Error> {
        let oracle_address: Address = env.storage().instance()
            .get(&symbol_short!("oracle"))
            .ok_or(Error::OracleError)?;

        let oracle = reflector_contract::Client::new(env, &oracle_address);
        
        let price_data = oracle.lastprice(collateral_asset)
            .ok_or(Error::OracleError)?;

        let decimals = oracle.decimals();
        let collateral_value = (collateral_amount * price_data.price) / 10i128.pow(decimals);
        
        // Require 150% collateralization
        if collateral_value < (loan_amount * 150) / 100 {
            return Err(Error::InsufficientCollateral);
        }

        Ok(())
    }

    fn get_next_loan_id(env: &Env) -> u32 {
        let key = symbol_short!("new_count");
        let count: u32 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(count + 1));
        count
    }
}


mod test;