use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

use crate::{
    errors::{ApprovalError, MintError, TransferError},
    types::{Account, ApprovalArgs, Blob, CollectionMetadata, Metadata, TransferArgs},
};

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash)]
pub struct Approval {
    pub account: Account,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Token {
    pub id: Nat,
    pub name: String,
    pub description: String,
    pub image: Option<Blob>,
    pub owner: Account,
    pub approvals: HashSet<Approval>,
}

impl Token {
    fn approval_check(&self, account: &Account) -> bool {
        let current_time = ic_cdk::api::time();
        self.approvals.iter().any(|approval| {
            approval.account == *account
                && (approval.expires_at.is_none() || approval.expires_at >= Some(current_time))
        })
    }

    fn transfer(&mut self, caller: &Account, to: Account) -> Result<(), TransferError> {
        if self.owner == *caller || self.approval_check(&caller) {
            self.owner = to;
            self.approvals.clear();
            Ok(())
        } else {
            Err(TransferError::Unauthorized {
                tokens_ids: vec![self.id.clone()],
            })
        }
    }

    fn approve(&mut self, caller: &Account, approval: Approval) -> Result<(), ApprovalError> {
        if self.owner == *caller {
            self.approvals.insert(approval);
            Ok(())
        } else {
            Err(ApprovalError::Unauthorized {
                tokens_ids: vec![self.id.clone()],
            })
        }
    }

    fn metadata(&self) -> Vec<(String, Metadata)> {
        let mut metadata = vec![
            ("Name".into(), Metadata::Text(self.name.clone())),
            (
                "Description".into(),
                Metadata::Text(self.description.clone()),
            ),
        ];
        if let Some(ref image) = self.image {
            metadata.push(("Image".to_string(), Metadata::Blob(image.clone())));
        }
        metadata
    }
}

#[derive(CandidType, Deserialize)]
pub struct Collection {
    // name of the collection
    pub name: String,
    // symbol of the collection
    pub symbol: String,
    pub royalties: Option<u16>,
    // minting authority
    pub minting_authority: Principal,
    pub royalty_recipient: Option<Account>,
    pub description: Option<String>,
    pub image: Option<Blob>,
    pub total_supply: Nat,
    // max supply cap
    pub supply_cap: Option<Nat>,
    pub tokens: HashMap<Nat, Token>,
    pub tx_count: Nat,
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            name: String::new(),
            symbol: String::new(),
            royalties: None,
            minting_authority: Principal::anonymous(),
            royalty_recipient: None,
            description: None,
            image: None,
            total_supply: Nat::from(0),
            supply_cap: None,
            tokens: HashMap::new(),
            tx_count: Nat::from(0),
        }
    }
}

impl Collection {
    fn get_tx_id(&mut self) -> Nat {
        self.tx_count += Nat::from(1);
        self.tx_count.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn royalties(&self) -> Option<u16> {
        self.royalties.clone()
    }

    pub fn royalty_recipient(&self) -> Option<Account> {
        self.royalty_recipient.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn image(&self) -> Option<Blob> {
        self.image.clone()
    }

    pub fn total_supply(&self) -> Nat {
        self.total_supply.clone()
    }

    pub fn supply_cap(&self) -> Option<Nat> {
        self.supply_cap.clone()
    }

    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            icrc7_name: self.name.clone(),
            icrc7_symbol: self.symbol.clone(),
            icrc7_royalties: self.royalties.clone(),
            icrc7_royalty_recipient: self.royalty_recipient.clone(),
            icrc7_description: self.description.clone(),
            icrc7_image: self.image.clone(),
            icrc7_total_supply: self.total_supply.clone(),
            icrc7_supply_cap: self.supply_cap.clone(),
        }
    }

    pub fn mint(&mut self, caller: &Principal, token: Token) -> Result<Nat, MintError> {
        if *caller != self.minting_authority {
            return Err(MintError::Unauthorized {
                minting_authority: self.minting_authority.clone(),
            });
        }
        if let Some(ref cap) = self.supply_cap {
            if self.total_supply >= *cap {
                return Err(MintError::SupplyCapReached);
            }
        }
        self.total_supply += Nat::from(1);
        self.tokens.insert(token.id.clone(), token);
        Ok(self.get_tx_id())
    }

    pub fn owner_of(&self, id: &Nat) -> Account {
        match self.tokens.get(id) {
            None => ic_cdk::trap("Invalid Token Id"),
            Some(token) => token.owner.clone(),
        }
    }

    pub fn tokens_of(&self, account: &Account) -> Vec<Nat> {
        let mut ids = vec![];
        for (id, token) in self.tokens.iter() {
            if token.owner == *account {
                ids.push(id.clone())
            }
        }
        ids
    }

    pub fn token_metadata(&self, id: &Nat) -> Vec<(String, Metadata)> {
        match self.tokens.get(id) {
            Some(token) => token.metadata(),
            None => ic_cdk::trap("Invalid Id"),
        }
    }

    pub fn balance_of(&self, account: &Account) -> Nat {
        let mut balance = Nat::from(0);
        for (_, token) in self.tokens.iter() {
            if token.owner == *account {
                balance += Nat::from(1);
                continue;
            }
        }
        balance
    }

    pub fn check_approval(&self, id: &Nat, account: &Account) -> bool {
        match self.tokens.get(id) {
            None => ic_cdk::trap("Invalid Token Id"),
            Some(token) => token.approval_check(account),
        }
    }

    pub fn transfer(
        &mut self,
        caller: &Principal,
        arg: TransferArgs,
    ) -> Result<Nat, TransferError> {
        let auth = match arg.from {
            Some(from) => from,
            None => Account::from_principal(caller),
        };
        if let Some(time) = arg.created_at_time {
            let current_time = ic_cdk::api::time();
            if time < current_time {
                return Err(TransferError::TooOld);
            } else if time > current_time {
                return Err(TransferError::CreatedInFuture {
                    ledger_time: current_time,
                });
            }
        }
        let user_tokens = self.tokens_of(&auth);
        if arg.token_ids.len() == 0 {
            return Err(TransferError::GenericError {
                error_code: Nat::from(2),
                msg: "token_ids can't be empty".into(),
            });
        }
        if let Some(true) | None = arg.is_atomic {
            let mut unauthorized = vec![];
            for id in arg.token_ids.iter() {
                if user_tokens.contains(id) || self.check_approval(id, &auth) {
                    continue;
                } else {
                    unauthorized.push(id.clone())
                }
            }
            if unauthorized.len() > 0 {
                return Err(TransferError::Unauthorized {
                    tokens_ids: unauthorized,
                });
            }
        }
        for id in arg.token_ids {
            self.tokens
                .get_mut(&id)
                .unwrap()
                .transfer(&auth, arg.to.clone())?;
        }
        Ok(self.get_tx_id())
    }

    pub fn approve(&mut self, caller: &Principal, arg: ApprovalArgs) -> Result<Nat, ApprovalError> {
        let caller = Account {
            owner: caller.clone(),
            subaccount: arg.from_subaccount,
        };
        let user_tokens = self.tokens_of(&caller);
        let tokens = match arg.tokenIds {
            Some(ids) => {
                let mut unauthorized = vec![];
                for id in ids.iter() {
                    if user_tokens.contains(id) {
                        continue;
                    } else {
                        unauthorized.push(id.clone())
                    }
                }
                if unauthorized.len() > 0 {
                    return Err(ApprovalError::Unauthorized {
                        tokens_ids: unauthorized,
                    });
                } else {
                    ids
                }
            }
            None => user_tokens,
        };
        for token in tokens.iter() {
            let approval = Approval {
                account: Account::from_principal(&arg.to),
                expires_at: arg.expires_at,
            };
            self.tokens
                .get_mut(token)
                .unwrap()
                .approve(&caller, approval)
                .map_err(|e| e)?;
        }
        Ok(self.get_tx_id())
    }
}

thread_local! {
    pub static COLLECTION: RefCell<Collection> = RefCell::default();
}
