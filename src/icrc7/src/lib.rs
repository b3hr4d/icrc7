pub mod errors;
pub mod state;
pub mod types;

use crate::types::{CollectionMetadata, Standard};
use crate::{
    errors::{ApprovalError, TransferError},
    state::Token,
    state::{Config, CONFIG},
    types::{ApprovalArgs, MintArgs, TransferArgs},
};
use b3_utils::ledger::{ICRC1MetadataValue, ICRCAccount};
use ic_cdk::{init, query, update};
use state::TOTAL_SUPPLY;

#[init]
pub fn init(arg: Config) {
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();

        c.set(arg).unwrap();
    });
}

/// ======== Query ========

#[query]
pub fn icrc7_name() -> String {
    CONFIG.with(|c| c.borrow().get().name())
}

#[query]
pub fn icrc7_symbol() -> String {
    CONFIG.with(|c| c.borrow().get().symbol())
}

#[query]
pub fn icrc7_royalties() -> Option<u16> {
    CONFIG.with(|c| c.borrow().get().royalties())
}

#[query]
pub fn icrc7_royalty_recipient() -> Option<ICRCAccount> {
    CONFIG.with(|c| c.borrow().get().royalty_recipient())
}

#[query]
pub fn icrc7_description() -> Option<String> {
    CONFIG.with(|c| c.borrow().get().description())
}

#[query]
pub fn icrc7_image() -> Option<String> {
    CONFIG.with(|c| c.borrow().get().image())
}

#[query]
pub fn icrc7_total_supply() -> u128 {
    TOTAL_SUPPLY.with(|s| s.borrow().get().clone())
}

#[query]
pub fn icrc7_supply_cap() -> Option<u128> {
    CONFIG.with(|c| c.borrow().get().supply_cap())
}

#[query]
pub fn icrc7_collection_metadata() -> CollectionMetadata {
    CONFIG.with(|c| c.borrow().get().metadata())
}

#[query]
pub fn icrc7_metadata(id: u128) -> Vec<(String, ICRC1MetadataValue)> {
    CONFIG.with(|c| c.borrow().get().token_metadata(&id))
}

#[query]
pub fn icrc7_owner_of(id: u128) -> ICRCAccount {
    CONFIG.with(|collection| collection.borrow().get().owner_of(&id))
}

#[query]
pub fn icrc7_balance_of(account: ICRCAccount) -> u128 {
    CONFIG.with(|collection| collection.borrow().get().balance_of(&account))
}

#[query]
pub fn icrc7_tokens_of(account: ICRCAccount) -> Vec<u128> {
    CONFIG.with(|collection| collection.borrow().get().tokens_of(&account))
}

#[query]
pub fn icrc7_supported_standards() -> Vec<Standard> {
    vec![Standard {
        name: "ICRC-7".into(),
        url: "https://github.com/dfinity/ICRC/ICRCs/ICRC-7".into(),
    }]
}

// ======== Update ========

#[update]
pub fn icrc7_transfer(arg: TransferArgs) -> Result<u128, TransferError> {
    let caller = ic_cdk::caller();
    CONFIG.with(|collection| collection.borrow().get().transfer(&caller, arg))
}

#[update]
pub fn icrc7_approve(arg: ApprovalArgs) -> Result<u128, ApprovalError> {
    let caller = ic_cdk::caller();
    CONFIG.with(|collection| collection.borrow().get().approve(&caller, arg))
}

#[update]
pub fn icrc7_mint(arg: MintArgs) -> u128 {
    let caller = ic_cdk::caller();

    let token = Token {
        id: arg.id,
        name: arg.name,
        description: arg.description,
        image: arg.image,
        owner: arg.to,
        approvals: Vec::new(),
    };

    CONFIG.with(|c| c.borrow().get().mint(&caller, token))
}

ic_cdk::export_candid!();
