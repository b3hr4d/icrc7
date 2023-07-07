#![allow(non_snake_case)]

use candid::{Principal, CandidType, Deserialize, Encode, Decode};
use ic_stable_structures::{Storable, BoundedStorable};
use serde::Serialize;

pub type Blob = Vec<u8>;

pub type Subaccount = [u8; 32];

#[derive(CandidType, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

impl Storable for Account{
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }
    
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Account{
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 100;
}

impl Account{
    pub fn from_principal(principal: &Principal) -> Account{
        Self { owner: principal.clone(), subaccount: None }
    }
}

#[derive(CandidType)]
pub struct CollectionMetadata {
    pub icrc7_name: String,
    pub icrc7_symbol: String,
    pub icrc7_royalties: Option<u16>,
    pub icrc7_royalty_recipient: Option<Account>,
    pub icrc7_description: Option<String>,
    pub icrc7_image: Option<Blob>, // TBD
    pub icrc7_total_supply: u128,
    pub icrc7_supply_cap: Option<u128>,
}

#[derive(CandidType)]
pub enum Metadata{
    Nat(u128),
    Int(i128),
    Text(String),
    Blob(Blob),
}

#[derive(CandidType)]
pub struct Standard{
    pub name: String,
    pub url: String,
}

#[derive(CandidType, Deserialize)]
pub struct TransferArgs{
    pub from: Option<Account>,
    pub to: Account,
    pub token_ids: Vec<u128>,
    pub memo: Option<Blob>,
    pub created_at_time: Option<u64>,
    pub is_atomic: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct ApprovalArgs{
    pub from_subaccount: Option<Subaccount>,
    pub to: Principal,
    pub tokenIds: Option<Vec<u128>>,
    pub expires_at: Option<u64>,
    pub memo: Option<Blob>,
    pub created_at: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct MintArgs{
    pub id: u128,
    pub name: String,
    pub description: String,
    pub image: Option<Blob>,
    pub to: Account,
}

#[derive(CandidType, Deserialize)]
pub struct InitArg{
    pub name: String,
    pub symbol: String,
    pub minting_authority: Option<Principal>,
    pub royalties: Option<u16>,
    pub royalties_recipient: Option<Account>,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
    pub supply_cap: Option<u128>,
}

// #[derive(CandidType, Deserialize)]
// pub enum Transaction{
//     Mint{
//         time: u64,
//     },
//     Transfer{
//         time: u64,
//     },
//     Approval{
//         time: u64,
//     },
// }