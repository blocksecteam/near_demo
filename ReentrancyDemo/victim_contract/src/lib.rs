use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen,ext_contract,PromiseResult,Promise};


pub const FT_TOKEN: &str = "ft_token.test.near";
const GAS_FOR_SINGLE_CALL: u64 = 20000000000000;


#[ext_contract(ext_self)]
pub trait FTResolver {
    fn ft_resolve_transfer(
        &mut self,
        amount: u128
    ) -> bool;
}


#[ext_contract]
pub trait ExtFtToken {
    fn ft_transfer_call(&mut self,amount: u128)-> u128;
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct VictimContract {
    attacker_balance: u128,
    other_balance: u128,
}

impl Default for VictimContract {
    fn default() -> Self {
        Self {
            attacker_balance: 100,
            other_balance:100
        }
    }
}



#[near_bindgen]
impl VictimContract {
    pub fn withdraw(&mut self,amount: u128) -> Promise{
        assert!(self.attacker_balance>= amount);
        // Call Attacker的收币函数
        ext_ft_token::ft_transfer_call(
            amount.into(), 
            &FT_TOKEN, 
            0, 
            env::prepaid_gas() - GAS_FOR_SINGLE_CALL * 2
            )
            .then(ext_self::ft_resolve_transfer(
                amount.into(),
                &env::current_account_id(),
                0,
                GAS_FOR_SINGLE_CALL,
            ))
    }

    pub fn view_attacker_balance(&self) -> u128{
        self.attacker_balance
    }
    


    #[private]
    pub fn ft_resolve_transfer(&mut self, amount: u128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.attacker_balance -= amount;
            }
            PromiseResult::Failed => {

            }
        };
    }
}



