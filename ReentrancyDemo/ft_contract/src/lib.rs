use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen,ext_contract,PromiseOrValue};
use near_sdk::json_types::U128;

pub const ATTACKER: &str = "attacker.test.near";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

#[ext_contract(ext_fungible_token_receiver)]
pub trait FungibleTokenReceiver  {
    fn ft_on_transfer(&mut self, amount: u128)-> u128;
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    attacker_balance: u128,
    victim_balance: u128

}

impl Default for FungibleToken {
    fn default() -> Self {
        Self {
            attacker_balance: 0,
            victim_balance: 200
        }
    }
}

#[near_bindgen]
impl FungibleToken {
    pub fn ft_transfer_call(&mut self,amount: u128)-> PromiseOrValue<U128>{
        // 相当于 internal_ft_transfer
        self.attacker_balance += amount;
        self.victim_balance   -= amount;

        // Call Attacker的收币函数
        ext_fungible_token_receiver::ft_on_transfer(
            amount.into(), 
            &ATTACKER,
            0, 
            env::prepaid_gas() - GAS_FOR_SINGLE_CALL
            ).into()
    }

    pub fn view_attacker_balance(&self) -> u128{
        self.attacker_balance
    }

    pub fn view_victim_balance(&self) -> u128{
        self.victim_balance
    }
}
