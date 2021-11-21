use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::*;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey, PromiseOrValue,
};

const FTTOKEN: &str = "ft_token.test.near";
const DEFAULT_ACCOUNT: &str = "default.test.near";
const ERR_REFUND: &str = "Cannot Refund";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// AccountID -> Account balance.
    pub registered: Vec<AccountId>,
    pub bid_price: UnorderedMap<AccountId, Balance>,
    pub current_leader: AccountId,
    pub highest_bid: u128,
    pub refund: bool,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            registered: Default::default(),
            bid_price: UnorderedMap::new(StorageKey::BidPrice),
            current_leader: DEFAULT_ACCOUNT.to_string(),
            highest_bid: 0,
            refund: false,
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    BidPrice,
}

#[ext_contract]
pub trait ExtFtToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: u128);
    fn account_exist(&self, account_id: AccountId);
}

#[near_bindgen]
impl Contract {
    pub fn register_account(&mut self) {
        if self
            .bid_price
            .insert(&env::predecessor_account_id(), &0)
            .is_some()
        {
            env::panic("The account is already registered".to_string().as_bytes());
        } else {
            self.registered.push(env::predecessor_account_id());
        }
        log!("Registered account {}", env::predecessor_account_id());
    }

    pub fn bid(&mut self, sender_id: AccountId, amount: u128) -> PromiseOrValue<u128> {
        assert!(amount > self.highest_bid);
        self.refund_exe();
        self.current_leader = sender_id;
        self.highest_bid = amount;
        log!(
            "current_leader: {} highest_bid: {}",
            self.current_leader,
            self.highest_bid
        );
        PromiseOrValue::Value(0)
    }

    pub(crate) fn refund_exe(&mut self) {
        if self.current_leader == DEFAULT_ACCOUNT {
            return;
        } else {
            ext_ft_token::account_exist(
                self.current_leader.clone(),
                &FTTOKEN,
                0,
                env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 3,
            );

            assert!(self.refund, "{}", ERR_REFUND);
            ext_ft_token::ft_transfer(
                self.current_leader.clone(),
                self.highest_bid,
                &FTTOKEN,
                0,
                GAS_FOR_SINGLE_CALL * 2,
            );
        }
    }

    // 直接调用没有意义
    pub fn abort_refund(&mut self) {
        self.refund = false;
    }

    pub fn promise_refund(&mut self) {
        self.refund = true;
    }

    pub fn view_current_leader(&mut self) -> AccountId {
        self.current_leader.clone()
    }

    pub fn view_highest_bid(&mut self) -> u128 {
        self.highest_bid
    }
}
