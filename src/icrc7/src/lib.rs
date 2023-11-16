pub mod errors;
pub mod memory;
pub mod state;
pub mod types;
pub mod utils;

use crate::{
    errors::{ApprovalError, TransferError},
    state::Token,
    state::{Collection, COLLECTION},
    types::InitArg,
    types::{ApprovalArgs, MintArgs, TransferArgs},
};
use candid::candid_method;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

#[init]
#[candid_method(init)]
pub fn init(arg: InitArg) {
    let authority = match arg.minting_authority {
        None => ic_cdk::caller(),
        Some(auth) => auth,
    };
    COLLECTION.with(|c| {
        let mut c = c.borrow_mut();
        let collection = Collection {
            name: arg.name,
            symbol: arg.symbol,
            royalties: arg.royalties,
            minting_authority: authority,
            royalty_recipient: arg.royalties_recipient,
            description: arg.description,
            image: arg.image,
            supply_cap: arg.supply_cap,
            tx_window: arg.tx_window as u64 * 60 * 60 * 60 * 1000_000_000,
            permitted_drift: arg.permitted_drift as u64 * 60 * 60 * 1000_000_000,
            ..Default::default()
        };
        *c = collection;
    });
    // TX_WINDOW.with(|window|{
    //     let time = arg.tx_window as u64 * 60 * 60 * 60 * 1000_000_000;
    //     *window.borrow_mut() = time;
    // });
    // PERMITTED_DRIFT.with(|drift|{
    //     let time = arg.permitted_drift as u64 * 60 * 60 * 1000_000_000;
    //     *drift.borrow_mut() = time;
    // });
}

// A pre-upgrade hook for serializing the data stored on the heap.
#[pre_upgrade]
fn pre_upgrade() {
    // Serialize the state.
    // This example is using CBOR, but you can use any data format you like.
    let mut state_bytes = vec![];
    COLLECTION
        .with(|s| ciborium::ser::into_writer(&*s.borrow(), &mut state_bytes))
        .expect("failed to encode state");

    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = crate::memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
}

// A post-upgrade hook for deserializing the data back into the heap.
#[post_upgrade]
fn post_upgrade() {
    let memory = crate::memory::get_upgrades_memory();

    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);

    // Deserialize and set the state.
    let state = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
    COLLECTION.with(|s| *s.borrow_mut() = state);
}

/// ======== Query ========
use icrc_ledger_types::{icrc::generic_metadata_value::MetadataValue, icrc1::account::Account};

use crate::{
    types::{CollectionMetadata, Standard},
    utils::account_transformer,
};

#[query]
#[candid_method(query)]
pub fn icrc7_name() -> String {
    COLLECTION.with(|c| c.borrow().name())
}

#[query]
#[candid_method(query)]
pub fn icrc7_symbol() -> String {
    COLLECTION.with(|c| c.borrow().symbol())
}

#[query]
#[candid_method(query)]
pub fn icrc7_royalties() -> Option<u16> {
    COLLECTION.with(|c| c.borrow().royalties())
}

#[query]
#[candid_method(query)]
pub fn icrc7_royalty_recipient() -> Option<Account> {
    COLLECTION.with(|c| c.borrow().royalty_recipient())
}

#[query]
#[candid_method(query)]
pub fn icrc7_description() -> Option<String> {
    COLLECTION.with(|c| c.borrow().description())
}

#[query]
#[candid_method(query)]
pub fn icrc7_image() -> Option<String> {
    COLLECTION.with(|c| c.borrow().image())
}

#[query]
#[candid_method(query)]
pub fn icrc7_total_supply() -> u128 {
    COLLECTION.with(|c| c.borrow().total_supply())
}

#[query]
#[candid_method(query)]
pub fn icrc7_supply_cap() -> Option<u128> {
    COLLECTION.with(|c| c.borrow().supply_cap())
}

#[query]
#[candid_method(query)]
pub fn icrc7_collection_metadata() -> CollectionMetadata {
    COLLECTION.with(|c| c.borrow().metadata())
}

#[query]
#[candid_method(query)]
pub fn icrc7_metadata(id: u128) -> Vec<(String, MetadataValue)> {
    COLLECTION.with(|c| c.borrow().token_metadata(&id))
}

#[query]
#[candid_method(query)]
pub fn icrc7_owner_of(id: u128) -> Account {
    COLLECTION.with(|collection| collection.borrow().owner_of(&id))
}

#[query]
#[candid_method(query)]
pub fn icrc7_balance_of(account: Account) -> u128 {
    let account = account_transformer(account);
    COLLECTION.with(|collection| collection.borrow().balance_of(&account))
}

#[query]
#[candid_method(query)]
pub fn icrc7_tokens_of(account: Account) -> Vec<u128> {
    let account = account_transformer(account);
    COLLECTION.with(|collection| collection.borrow().tokens_of(&account))
}

#[query]
#[candid_method(query)]
pub fn icrc7_supported_standards() -> Vec<Standard> {
    vec![Standard {
        name: "ICRC-7".into(),
        url: "https://github.com/dfinity/ICRC/ICRCs/ICRC-7".into(),
    }]
}

#[update]
#[candid_method(update)]
pub fn icrc7_transfer(arg: TransferArgs) -> Result<u128, TransferError> {
    let caller = ic_cdk::caller();
    COLLECTION.with(|c| {
        let mut c = c.borrow_mut();
        c.transfer(&caller, arg)
    })
}

#[update]
#[candid_method(update)]
pub fn icrc7_approve(arg: ApprovalArgs) -> Result<u128, ApprovalError> {
    let caller = ic_cdk::caller();
    COLLECTION.with(|c| {
        let mut c = c.borrow_mut();
        c.approve(&caller, arg)
    })
}

#[update]
#[candid_method(update)]
pub fn icrc7_mint(arg: MintArgs) -> u128 {
    let caller = ic_cdk::caller();
    let owner = account_transformer(arg.to);
    COLLECTION.with(|c| {
        let mut c = c.borrow_mut();
        let token = Token {
            id: arg.id,
            name: arg.name,
            description: arg.description,
            image: arg.image,
            owner,
            approvals: Vec::new(),
        };
        c.mint(&caller, token)
    })
}
