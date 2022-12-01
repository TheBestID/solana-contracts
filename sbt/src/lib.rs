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

type MyHash = [u8; 32];
type HashedData = [MyHash; 2];

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PayAccount {
    pub payer_pubkey: Pubkey,
    pub deposit: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Soul {
    soul_id: u128,
    git_hash: MyHash,
    email_hash: MyHash,
}

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct SBT {
    souls_: HashMap<u128, Soul>,
    soul_id_of_account_: HashMap<AccountId, u128>,
    account_of_soul_id: HashMap<u128, AccountId>,
    minted_not_claimed: HashMap<u128, bool>,
}

impl SBT {
    pub fn init() -> Self {
        Self {
            souls_: HashMap::new(),
            soul_id_of_account_: HashMap::new(),
            account_of_soul_id: HashMap::new(),
            minted_not_claimed: HashMap::new(),
        }
    }

    pub fn mint(&mut self, new_id: u128, account: &AccountId) {
        require!(!self.souls_.contains_key(&new_id), "Soul exists");
        require!(
            !self.minted_not_claimed.contains_key(&new_id),
            "Soul is already minted"
        );
        require!(
            env::signer_account_id() == env::current_account_id(),
            "Only this contract can mint SBT"
        );
        *self.soul_id_of_account_.get_mut(account).unwrap() = new_id;
        *self.minted_not_claimed.get_mut(&new_id).unwrap() = true;
    }

    pub fn claim(&mut self, new_git_hash: &MyHash, new_email_hash: &MyHash) {
        let msg_sender = env::signer_account_id();
        require!(
            self.soul_id_of_account_.contains_key(&msg_sender),
            "Soul must exist to be able to burn it"
        );
        let msg_sender_soul_id = self.soul_id_of_account_[&msg_sender];
        require!(
            self.minted_not_claimed.contains_key(&msg_sender_soul_id)
                && self.minted_not_claimed[&msg_sender_soul_id],
            "Soul must be minted to be able to claim it"
        );
        self.souls_.get_mut(&msg_sender_soul_id).unwrap().email_hash = *new_email_hash;
        self.souls_.get_mut(&msg_sender_soul_id).unwrap().git_hash = *new_git_hash;
        *self
            .account_of_soul_id
            .get_mut(&msg_sender_soul_id)
            .unwrap() = msg_sender;
        self.minted_not_claimed.remove(&msg_sender_soul_id);
    }

    pub fn get_user_id(&self, account: &AccountId) -> u128 {
        require!(
            self.soul_id_of_account_.contains_key(account),
            "No user found"
        );
        self.soul_id_of_account_[account]
    }

    pub fn get_account_id(&self, user_id: u128) -> AccountId {
        require!(
            self.souls_.contains_key(&user_id),
            "No user found"
        );
        self.account_of_soul_id[&user_id].clone()
    }


    pub fn burn(&mut self) {
        let msg_sender = env::signer_account_id();
        require!(
            self.soul_id_of_account_.contains_key(&msg_sender),
            "Soul must exist to be able to burn it"
        );
        let msg_sender_soul_id = self.soul_id_of_account_[&msg_sender];
        require!(
            self.souls_.contains_key(&msg_sender_soul_id),
            "Soul must exist to be able to burn it"
        );
        self.soul_id_of_account_.remove(&msg_sender);
        self.account_of_soul_id.remove(&msg_sender_soul_id);
        self.souls_.remove(&msg_sender_soul_id);
    }

    pub fn has_soul(&self, account: &AccountId) -> bool {
        if !self.soul_id_of_account_.contains_key(account) {
            return false;
        }
        if !self.souls_.contains_key(&self.soul_id_of_account_[account]) {
            return false;
        }
        true
    }

    pub fn ping(&self) -> bool {
        true
    }

    pub fn ping_string(&self) -> String {
        "I'm okey".to_string()
    }

    pub fn get_hashed_data(&self) -> HashedData {
        let msg_sender = env::signer_account_id();
        let msg_sender_soul_id = self.soul_id_of_account_[&msg_sender];
        require!(
            self.account_of_soul_id[&msg_sender_soul_id] == msg_sender,
            "Soul must exist to get it's data"
        );
        let mut users_data = [[0; 32]; 2];

        users_data[0] = self.souls_[&msg_sender_soul_id].git_hash;
        users_data[1] = self.souls_[&msg_sender_soul_id].email_hash;
        users_data
    }
}

use crate::SBT::init;

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) {}
