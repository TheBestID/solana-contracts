// SPDX-License-Identifier: MIT
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::collections::HashMap;
use std::vec::Vec;



#[ext_contract(sbt_contract)]
trait SBTContractInterface { 
    fn get_user_id(&self, _user: AccountId) -> u128;
    fn get_account_id(&self,user_id: u128) -> AccountId;
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Achievement {
    achievement_id: u128,
    achievement_type: u128,
    issuer: u128,
    owner: u128,
    is_accepted: bool,
    verifier: u128,
    is_verified: bool,
    data_address: String,
    balance: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PayAccount {
    pub payer_pubkey: Pubkey,
    pub deposit: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AchievementToken {
    achievements: HashMap<u128, Achievement>,
    issuers_achievements: HashMap<u128, Vec<u128>>,
    users_achievements: HashMap<u128, Vec<u128>>,
}

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);


impl AchievementToken {
    fn get_user_id(&self, user: AccountId, id : u128) -> Promise {
        let sbt_account: AccountId = "sbt.soul_dev.testnet".parse().unwrap();
        sbt_contract::ext(sbt_account)
        .get_user_id(user)
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .get_user_id_callback(id)
        )
    }
    

    #[private]
    pub fn get_user_id_callback(&self, id : u128, #[callback_result] call_result: Result<u128, PromiseError>) -> bool {
        if call_result.is_err() {
            log!("There was an error contacting SBT contract");
            return false;
        }

        let uid: u128 = call_result.unwrap();
        require!(id == uid, "Wrong uid");
        true
    }

    fn transfer_to_verifier(&self, user_id: u128, money : u128) -> Promise {
        let sbt_account: AccountId = "sbt.soul_dev.testnet".parse().unwrap();
        sbt_contract::ext(sbt_account)
        .get_account_id(user_id)
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .transfer_to_verifier_callback(money)
        )
    }
    

    #[private]
    pub fn transfer_to_verifier_callback(&self, money : u128, #[callback_result] call_result: Result<AccountId, PromiseError>) -> bool {
        if call_result.is_err() {
            log!("There was an error contacting SBT contract");
            return false;
        }

        let account: AccountId = call_result.unwrap();
        // require!(id == uid, "Wrong uid");
        Promise::new(account).transfer(money);
        true
    }

    #[payable]
    fn mint(&mut self, _achievement_data: Achievement) {
        require!(
            self.achievements[&_achievement_data.achievement_id].issuer != 0,
            "Achievement id already exists"
        );
        self.get_user_id(env::signer_account_id(), _achievement_data.issuer);
        require!(
            env::signer_account_id() == env::current_account_id(),
            "Only you can be an issuer"
        );
        require!(
            env::attached_deposit() >= _achievement_data.balance,
            "Not enough deposit attached"
        );

        let return_money = env::attached_deposit() - _achievement_data.balance;
        Promise::new(env::signer_account_id()).transfer(return_money);

        let current_id = _achievement_data.achievement_id;
        *self.achievements.get_mut(&current_id).unwrap() = _achievement_data;
        self.issuers_achievements
            .get_mut(&self.achievements[&current_id].issuer)
            .unwrap()
            .push(current_id);
        self.users_achievements
            .get_mut(&self.achievements[&current_id].owner)
            .unwrap()
            .push(current_id);
    }

    fn burn(&mut self, _achievement_id: u128) {
        require!(
            self.achievements[&_achievement_id].issuer != 0,
            "Achievement id already exists"
        );
        self.get_user_id(env::signer_account_id(), self.achievements[&_achievement_id].issuer);
        let current_issuer_id = self.achievements[&_achievement_id].issuer;
        for i in 0..self.issuers_achievements[&current_issuer_id].len() {
            if _achievement_id == self.issuers_achievements[&current_issuer_id][i] {
                self.issuers_achievements
                    .get_mut(&current_issuer_id)
                    .unwrap()
                    .remove(i);
                break;
            }
        }

        let current_owner_id = self.achievements[&_achievement_id].owner;
        for i in 0..self.users_achievements[&current_owner_id].len() {
            if _achievement_id == self.users_achievements[&current_owner_id][i] {
                self.users_achievements
                    .get_mut(&current_owner_id)
                    .unwrap()
                    .remove(i);
                break;
            }
        }  
        self.achievements.remove(&_achievement_id);      
    }

    fn set_new_owner(&mut self, user: AccountId, id : u128) -> Promise {
        let sbt_account: AccountId = "sbt.soul_dev.testnet".parse().unwrap();
        sbt_contract::ext(sbt_account)
        .get_user_id(user)
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .get_user_id_callback(id)
        )
    }
    

    #[private]
    pub fn set_new_owner_callback(&mut self, _achievement_id: u128, #[callback_result] call_result: Result<u128, PromiseError>) -> bool {
        if call_result.is_err() {
            log!("There was an error contacting SBT contract");
            return false;
        }

        let user_id: u128 = call_result.unwrap();
        self.achievements.get_mut(&_achievement_id).unwrap().owner = user_id;
        // require!(id == uid, "Wrong uid");
        true
    } 

    fn update_owner(&mut self, _achievement_id: u128, _new_owner: AccountId) {
        // let new_account_id = get_user_id(_new_owner);
        self.set_new_owner(_new_owner, _achievement_id);
        self.get_user_id(env::signer_account_id(), self.achievements[&_achievement_id].issuer);
        require!(
            self.achievements[&_achievement_id].owner == 0,
            "Owner of this achievement can not be changed"
        );
    }

    fn accept_achievement(&mut self, _achievement_id: u128) {
        self.get_user_id(env::signer_account_id(), self.achievements[&_achievement_id].owner);
        self.achievements.get_mut(&_achievement_id).unwrap().is_accepted = true;
    }

    fn verify_achievement(&mut self, _achievement_id: u128) {
        let current_verifier = self.achievements[&_achievement_id].verifier;
        self.get_user_id(env::signer_account_id(), current_verifier);
        require!(
            !self.achievements[&_achievement_id].is_verified,
            "Achievement already verified"
        );
        self.transfer_to_verifier(current_verifier, self.achievements[&_achievement_id].balance);
    }

    // fn split_achievement(&mut self, _achievement_id: u128) {
// TODO: Do we actually need it?
    // }

    fn get_achievement_data(&self, _achievement_id: u128) -> Achievement {
        self.achievements[&_achievement_id].clone()
    }

    #[payable]
    fn replenish_achievement_balance(&mut self, _achievement_id: u128) {
        self.achievements.get_mut(&_achievement_id).unwrap().balance += env::attached_deposit();
    }

}

use crate::AchievementToken::init;

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) {}