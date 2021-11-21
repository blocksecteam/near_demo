use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::*;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey};

pub const FTTOKEN: &str = "ft_token.test.near";
const DISTRIBUTOR: &str = "user0.test.near";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// AccountID -> Account balance.
    pub registered: Vec<AccountId>,
    pub accounts: UnorderedMap<AccountId, Balance>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            registered: Default::default(),
            accounts: UnorderedMap::new(StorageKey::Accounts),
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
}

#[ext_contract]
pub trait ExtFtToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: u128);
}

#[near_bindgen]
impl Contract {
    pub fn register_account(&mut self) {
        if self
            .accounts
            .insert(&env::predecessor_account_id(), &0)
            .is_some()
        {
            env::panic("The account is already registered".to_string().as_bytes());
        } else {
            self.registered.push(env::predecessor_account_id());
        }
        log!("Registered account{}", env::predecessor_account_id());
    }

    pub fn distribute_token(&mut self, amount: u128) {
        assert_eq!(
            env::predecessor_account_id(),
            DISTRIBUTOR,
            "ERR_NOT_ALLOWED"
        );
        for cur_account in self.registered.iter() {
            let balance = self.accounts.get(&cur_account).expect("ERR_GET");
            self.accounts
                .insert(&cur_account, &balance.checked_add(amount).expect("ERR_ADD"));
            log!("Try distribute to account{}", &cur_account);
            ext_ft_token::ft_transfer(
                cur_account.clone(),
                amount,
                &FTTOKEN,
                0,
                GAS_FOR_SINGLE_CALL,
            );
        }
    }
}
