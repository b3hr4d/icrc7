use std::{cell::RefCell, collections::HashMap};

use b3_utils::{
    ledger::{ICRC1MetadataValue, ICRCAccount},
    memory::{
        init_stable_mem_refcell,
        types::{Bound, DefaultStableBTreeMap, DefaultStableCell, DefaultStableVec, Storable},
    },
};
use candid::{CandidType, Decode, Encode, Nat, Principal};
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::{
    errors::{ApprovalError, TransferError},
    types::{ApprovalArgs, CollectionMetadata, TransferArgs},
};

thread_local! {
    pub static CONFIG: RefCell<DefaultStableCell<Config>> = init_stable_mem_refcell("config", 1).unwrap();
    pub static TOKENS: RefCell<DefaultStableBTreeMap<u128, Token>> = init_stable_mem_refcell("tokens", 2).unwrap();
    pub static TRANSFER_LOG: RefCell<DefaultStableVec<TransferLog>> = init_stable_mem_refcell("transfer_log", 3).unwrap();
    pub static TRANSACTION_ID: RefCell<DefaultStableCell<u128>> = init_stable_mem_refcell("transaction_id", 4).unwrap();
    pub static TOTAL_SUPPLY: RefCell<DefaultStableCell<u128>> = init_stable_mem_refcell("total_supply", 5).unwrap();
}

fn increment_tx_id() {
    TRANSACTION_ID.with(|id| {
        let mut id = id.borrow_mut();
        let current_id = id.get().clone();

        id.set(current_id + 1).unwrap();
    })
}

fn get_tx_id() -> u128 {
    TRANSACTION_ID.with(|id| id.borrow().get().clone())
}

fn increment_total_supply() {
    TOTAL_SUPPLY.with(|s| {
        let mut s = s.borrow_mut();
        let current_supply = s.get().clone();
        s.set(current_supply + 1).unwrap();
    })
}

fn get_total_supply() -> u128 {
    TOTAL_SUPPLY.with(|s| s.borrow().get().clone())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Approval {
    pub expires_at: Option<u64>,
    pub account: ICRCAccount,
}

impl Storable for Approval {
    const BOUND: Bound = Bound::Bounded {
        is_fixed_size: false,
        max_size: 100,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Approval {
    pub fn new(account: ICRCAccount, expires_at: Option<u64>) -> Self {
        Self {
            expires_at,
            account,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct Token {
    pub id: u128,
    pub owner: ICRCAccount,
    pub name: String,
    pub image: Option<Vec<u8>>,
    pub description: Option<String>,
    pub approvals: Vec<Approval>,
}

impl Token {
    pub fn token_metadata(&self) -> Vec<(String, ICRC1MetadataValue)> {
        let mut metadata = Vec::new();
        metadata.push((
            "Id".to_string(),
            ICRC1MetadataValue::Nat(Nat::from(self.id)),
        ));
        metadata.push(("Name".into(), ICRC1MetadataValue::Text(self.name.clone())));
        if self.image.is_some() {
            let buf = ByteBuf::from(self.image.as_ref().unwrap().clone());
            metadata.push(("Image".into(), ICRC1MetadataValue::Blob(buf)))
        }
        if self.description.is_some() {
            let value = self.description.as_ref().unwrap().clone();
            metadata.push(("Description".into(), ICRC1MetadataValue::Text(value)))
        }
        metadata
    }

    pub fn owner(&self) -> ICRCAccount {
        self.owner.clone()
    }

    pub fn approval_check(&self, current_time: u64, account: &ICRCAccount) -> bool {
        // self.approvals.iter().any(|approval| {
        //     ic_cdk::println!("owner: {:?} == {:?} && subaccount {:?} == {:?}", approval.account.owner, account.owner, approval.account.subaccount, account.subaccount);
        //     approval.account.owner == account.owner && approval.account.subaccount == account.subaccount
        //         && (approval.expires_at.is_none() || approval.expires_at >= Some(current_time))
        // })
        for approval in self.approvals.iter() {
            if approval.account == *account {
                if approval.expires_at.is_none() {
                    return true;
                } else if approval.expires_at >= Some(current_time) {
                    return true;
                }
            }
        }
        false
    }

    pub fn approve(
        &mut self,
        caller: &ICRCAccount,
        approval: Approval,
    ) -> Result<(), ApprovalError> {
        if self.owner == approval.account {
            ic_cdk::trap("Self Approve")
        }
        if *caller != self.owner {
            return Err(ApprovalError::Unauthorized {
                tokens_ids: vec![self.id],
            });
        } else {
            self.approvals.push(approval);
            Ok(())
        }
    }

    pub fn transfer(
        &mut self,
        permitted_time: u64,
        caller: &ICRCAccount,
        to: ICRCAccount,
    ) -> Result<(), TransferError> {
        if self.owner == to {
            ic_cdk::trap("Self Transfer")
        }
        if self.owner != *caller && !self.approval_check(permitted_time, caller) {
            return Err(TransferError::Unauthorized {
                tokens_ids: vec![self.id],
            });
        } else {
            self.owner = to;
            self.approvals.clear();
            return Ok(());
        }
    }
}

impl Storable for Token {
    const BOUND: Bound = Bound::Bounded {
        max_size: 100000,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct TransferLog {
    pub id: u128,
    pub at: u64,
    pub memo: Option<Vec<u8>>,
    pub from: ICRCAccount,
    pub to: ICRCAccount,
}

impl Storable for TransferLog {
    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub symbol: String,
    pub royalties: Option<u16>,
    pub minting_authority: Principal,
    pub royalty_recipient: Option<ICRCAccount>,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub supply_cap: Option<u128>,
    pub tx_window: u64,
    pub permitted_drift: u64,
}

impl Storable for Config {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            symbol: String::new(),
            royalties: None,
            minting_authority: Principal::anonymous(),
            royalty_recipient: None,
            description: None,
            logo: None,
            supply_cap: None,
            tx_window: 0,
            permitted_drift: 0,
        }
    }
}

impl Config {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn royalties(&self) -> Option<u16> {
        self.royalties.clone()
    }

    pub fn royalty_recipient(&self) -> Option<ICRCAccount> {
        self.royalty_recipient.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn image(&self) -> Option<String> {
        self.logo.clone()
    }

    pub fn supply_cap(&self) -> Option<u128> {
        self.supply_cap.clone()
    }

    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            icrc7_name: self.name.clone(),
            icrc7_symbol: self.symbol.clone(),
            icrc7_royalties: self.royalties.clone(),
            icrc7_royalty_recipient: self.royalty_recipient.clone(),
            icrc7_description: self.description.clone(),
            icrc7_image: self.logo.clone(),
            icrc7_total_supply: get_total_supply(),
            icrc7_supply_cap: self.supply_cap.clone(),
        }
    }

    pub fn mint(&self, caller: &Principal, token: Token) -> u128 {
        if *caller != self.minting_authority {
            ic_cdk::trap("Unauthorized Caller")
        }
        if let Some(cap) = self.supply_cap {
            if cap < get_total_supply() {
                ic_cdk::trap("Supply Cap Reached")
            }
        }

        if TOKENS.with(|tokens| tokens.borrow().contains_key(&token.id)) {
            ic_cdk::trap("Id Exist")
        }

        increment_total_supply();

        TOKENS.with(|tokens| tokens.borrow_mut().insert(token.id, token));

        increment_tx_id();

        get_tx_id()
    }

    pub fn owner_of(&self, id: &u128) -> ICRCAccount {
        TOKENS.with(|tokens| {
            let tokens = tokens.borrow();
            match tokens.get(id) {
                None => ic_cdk::trap("Invalid Token Id"),
                Some(token) => token.owner.clone(),
            }
        })
    }

    pub fn tokens_of(&self, account: &ICRCAccount) -> Vec<u128> {
        let mut ids = vec![];
        TOKENS.with(|tokens| {
            for (id, token) in tokens.borrow().iter() {
                if token.owner == *account {
                    ids.push(id.clone())
                }
            }
        });

        ids
    }

    pub fn token_metadata(&self, id: &u128) -> Vec<(String, ICRC1MetadataValue)> {
        match TOKENS.with(|tokens| tokens.borrow().get(id)) {
            None => ic_cdk::trap("Invalid Token Id"),
            Some(token) => token.token_metadata(),
        }
    }

    pub fn balance_of(&self, account: &ICRCAccount) -> u128 {
        let mut balance = 0;

        TOKENS.with(|tokens| {
            for (_, token) in tokens.borrow().iter() {
                if token.owner == *account {
                    balance += 1;
                    continue;
                }
            }
        });

        balance
    }

    pub fn tx_deduplication_check(
        &self,
        permitted_past_time: u64,
        created_at_time: u64,
        memo: &Option<Vec<u8>>,
        id: u128,
        caller: &ICRCAccount,
        to: &ICRCAccount,
    ) -> Option<usize> {
        TRANSFER_LOG.with(|log_ref| {
            log_ref.borrow().iter().position(|log| {
                log.at > permitted_past_time
                    && log.id == id
                    && log.at == created_at_time
                    && log.memo == *memo
                    && log.from == *caller
                    && log.to == *to
            })
        })
    }

    fn id_validity_check(&self, ids: &Vec<u128>) {
        let mut invalid_ids = vec![];

        TOKENS.with(|tokens| {
            for id in ids.iter() {
                match tokens.borrow().get(id) {
                    Some(_) => continue,
                    None => invalid_ids.push(id.clone()),
                }
            }
        });

        if invalid_ids.len() > 0 {
            let error_msg = format!("Invalid Token Ids: {:?}", invalid_ids);
            ic_cdk::trap(&error_msg)
        }
    }

    pub fn transfer(&self, caller: &Principal, arg: TransferArgs) -> Result<u128, TransferError> {
        if arg.token_ids.len() == 0 {
            ic_cdk::trap("No Token Provided")
        }
        // checking if the token for respective ids is available or not
        self.id_validity_check(&arg.token_ids);

        let caller = ICRCAccount::new(*caller, arg.spender_subaccount);

        let current_time = ic_cdk::api::time();
        let mut tx_deduplication: HashMap<u128, TransferError> = HashMap::new();
        if let Some(arg_time) = arg.created_at_time {
            let permitted_past_time = current_time - self.tx_window - self.permitted_drift;
            let permitted_future_time = current_time + self.permitted_drift;

            if arg_time < permitted_past_time {
                return Err(TransferError::TooOld);
            }
            if arg_time > permitted_future_time {
                return Err(TransferError::CreatedInFuture {
                    ledger_time: current_time,
                });
            }

            arg.token_ids.iter().for_each(|id| {
                if let Some(index) = self.tx_deduplication_check(
                    permitted_past_time,
                    arg_time,
                    &arg.memo,
                    *id,
                    &caller,
                    &arg.to,
                ) {
                    tx_deduplication.insert(
                        *id,
                        TransferError::Duplicate {
                            duplicate_of: index as u128,
                        },
                    );
                }
            });
        }
        let mut unauthorized: Vec<u128> = vec![];
        arg.token_ids.iter().for_each(|id| {
            let token = match TOKENS.with(|tokens| tokens.borrow().get(id)) {
                None => ic_cdk::trap("Invalid Id"),
                Some(token) => token,
            };

            let approval_check = token.approval_check(current_time + self.permitted_drift, &caller);
            if token.owner != caller && !approval_check {
                unauthorized.push(id.clone())
            }
        });

        match arg.is_atomic {
            // when atomic transfer is turned off
            Some(false) => {
                for id in arg.token_ids.iter() {
                    if let Some(e) = tx_deduplication.get(id) {
                        return Err(e.clone());
                    }
                    let mut token = TOKENS.with(|tokens| tokens.borrow().get(id).unwrap());

                    match token.transfer(
                        current_time + self.permitted_drift,
                        &caller,
                        arg.to.clone(),
                    ) {
                        Err(_) => continue,
                        Ok(_) => {
                            let log = TransferLog {
                                id: id.clone(),
                                at: current_time,
                                memo: arg.memo.clone(),
                                from: caller.clone(),
                                to: arg.to.clone(),
                            };
                            TOKENS.with(|tokens| tokens.borrow_mut().insert(id.clone(), token));

                            TRANSFER_LOG.with(|log_ref| log_ref.borrow_mut().push(&log).unwrap());
                        }
                    }
                }
                if unauthorized.len() > 0 {
                    return Err(TransferError::Unauthorized {
                        tokens_ids: unauthorized,
                    });
                }

                increment_tx_id();

                Ok(get_tx_id())
            }
            // default behaviour of atomic
            _ => {
                for (_, e) in tx_deduplication.iter() {
                    return Err(e.clone());
                }
                if unauthorized.len() > 0 {
                    return Err(TransferError::Unauthorized {
                        tokens_ids: unauthorized,
                    });
                }
                for id in arg.token_ids.iter() {
                    let mut token = TOKENS.with(|tokens| tokens.borrow().get(id).unwrap());
                    token.transfer(current_time + self.permitted_drift, &caller, arg.to.clone())?;
                    let log = TransferLog {
                        id: id.clone(),
                        at: current_time,
                        memo: arg.memo.clone(),
                        from: caller.clone(),
                        to: arg.to.clone(),
                    };
                    TOKENS.with(|tokens| tokens.borrow_mut().insert(id.clone(), token));
                    TRANSFER_LOG.with(|log_ref| log_ref.borrow_mut().push(&log).unwrap());
                }

                increment_tx_id();

                Ok(get_tx_id())
            }
        }
    }

    pub fn approve(&self, caller: &Principal, arg: ApprovalArgs) -> Result<u128, ApprovalError> {
        let caller = ICRCAccount::from(*caller);
        let token_ids = match arg.token_ids {
            None => self.tokens_of(&caller),
            Some(ids) => {
                self.id_validity_check(&ids);
                ids
            }
        };
        if token_ids.len() == 0 {
            ic_cdk::trap("No Tokens Available")
        }
        let approve_for = ICRCAccount::from(arg.spender);
        let approval = Approval {
            account: approve_for,
            expires_at: arg.expires_at,
        };

        TOKENS.with(|tokens| {
            for id in token_ids.iter() {
                let mut token = tokens.borrow().get(id).unwrap();
                token.approve(&caller, approval.clone())?;
                tokens.borrow_mut().insert(id.clone(), token);
            }

            increment_tx_id();

            Ok(get_tx_id())
        })
    }
}
