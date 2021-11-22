use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::*;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey, PromiseOrValue,PromiseResult
};

const FTTOKEN: &str = "ft_token.test.near";
const DEFAULT_ACCOUNT: &str = "default.test.near";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// AccountID -> Account balance.
    pub registered: Vec<AccountId>,
    pub bid_price: UnorderedMap<AccountId, Balance>,
    pub current_leader: AccountId,
    pub highest_bid: u128,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            registered: Default::default(),
            bid_price: UnorderedMap::new(StorageKey::BidPrice),
            current_leader: DEFAULT_ACCOUNT.to_string(),
            highest_bid: 0,
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

#[ext_contract(ext_self)]
pub trait FTResolver {
    fn account_resolve(
        &mut self,
        sender_id: AccountId,
        amount: u128
    ) -> bool;
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

        if self.current_leader == DEFAULT_ACCOUNT {
            self.current_leader = sender_id;
            self.highest_bid = amount;
        } else {
            ext_ft_token::account_exist(
                self.current_leader.clone(),
                &FTTOKEN,
                0,
                env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 4,
            ).then(ext_self::account_resolve(
                sender_id,
                amount,
                &env::current_account_id(),
                0,
                GAS_FOR_SINGLE_CALL * 3, 
            ));
        }
        log!(
            "current_leader: {} highest_bid: {}",
            self.current_leader,
            self.highest_bid
        );
        PromiseOrValue::Value(0)
    }

    pub fn view_current_leader(&mut self) -> AccountId {
        self.current_leader.clone()
    }

    pub fn view_highest_bid(&mut self) -> u128 {
        self.highest_bid
    }

    #[private]
    pub fn account_resolve(&mut self,sender_id: AccountId,amount: u128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                    // 退回上任出价最高者的token
                    ext_ft_token::ft_transfer(
                        self.current_leader.clone(),
                        self.highest_bid,
                        &FTTOKEN,
                        0,
                        GAS_FOR_SINGLE_CALL * 2,
                    );
                    self.current_leader = sender_id;
                    self.highest_bid = amount;
            }
            PromiseResult::Failed => {
                // 退回当前出价最高者的token
                ext_ft_token::ft_transfer(
                    sender_id.clone(),
                    amount,
                    &FTTOKEN,
                    0,
                    GAS_FOR_SINGLE_CALL * 2,
                );
                log!("Return Back Now");
            }
        };
    }
}
