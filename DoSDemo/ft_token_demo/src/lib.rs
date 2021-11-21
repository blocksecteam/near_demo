use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::*;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey, Promise,
};

const OWNER: &str = "ft_token_owner.test.near";
const BIDCONTRACT: &str = "bid_contract.test.near";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// AccountID -> Account balance.
    pub accounts: UnorderedMap<AccountId, Balance>,

    /// Total supply of the all token.
    pub total_supply: Balance,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            accounts: UnorderedMap::new(StorageKey::Accounts),
            total_supply: 1_0000_0000,
        }
    }
}

#[ext_contract(ext_fungible_token_receiver)]
pub trait ExtBidContract {
    fn bid(&mut self, sender_id: AccountId, amount: u128);
    fn abort_refund(&self);
    fn promise_refund(&self);
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
}

#[near_bindgen]
impl Contract {
    pub fn register_account(&mut self, account_id: AccountId) {
        if env::predecessor_account_id() == OWNER && self.accounts.get(&OWNER.to_string()).is_none()
        {
            self.accounts
                .insert(&env::predecessor_account_id(), &self.total_supply);
        } else if self.accounts.insert(&account_id, &0).is_some() {
            env::panic("The account is already registered".to_string().as_bytes());
        }
    }

    pub fn unregister_account(&mut self) {
        assert_eq!(
            self.view_accounts(env::predecessor_account_id()),
            0,
            "The balance of the account is not zero."
        );
        self.accounts.remove(&env::predecessor_account_id());
        if self.accounts.get(&env::predecessor_account_id()).is_none() {
            log!("The account is already unregistered");
        }
    }

    pub fn view_accounts(&self, account_id: AccountId) -> Balance {
        self.internal_unwrap_balance_of(&account_id)
    }

    pub fn account_exist(&self, account_id: AccountId) -> Promise {
        if self.accounts.get(&account_id).is_none() {
            ext_fungible_token_receiver::abort_refund(
                &BIDCONTRACT,
                0,
                env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 3,
            )
        } else {
            ext_fungible_token_receiver::promise_refund(
                &BIDCONTRACT,
                0,
                env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 3,
            )
        }
    }

    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: u128) {
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, &receiver_id, amount);
    }

    pub fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: u128) -> Promise {
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, &receiver_id, amount);
        // Initiating receiver's call and the callback
        ext_fungible_token_receiver::bid(
            sender_id.clone(),
            amount.into(),
            &receiver_id.clone(),
            0,
            env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 2,
        )
    }
}

// This methods is still not exported.
// Another way of not exporting methods is by having a separate impl Contract section, that is not marked with #[near_bindgen].
impl Contract {
    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );
        assert!(amount > 0, "The amount should be a positive number");
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        log!("Transfer {} from {} to {}", amount, sender_id, receiver_id);
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.accounts.insert(&account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .expect("Total supply overflow");
        } else {
            env::panic(b"The account doesn't have enough balance");
        }
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(&account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .expect("Total supply overflow");
        } else {
            env::panic(b"Balance overflow");
        }
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        match self.accounts.get(&account_id) {
            Some(balance) => balance,
            None => env::panic(format!("The account {} is not registered", &account_id).as_bytes()),
        }
    }
}
