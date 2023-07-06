use ic_cdk_macros::{init, pre_upgrade, post_upgrade};
use candid::candid_method;

use crate::{types::InitArg, state::{COLLECTION, Collection}};

#[init]
#[candid_method(init)]
pub fn init(
    arg: InitArg
){
    let authority = match arg.minting_authority{
        None => ic_cdk::caller(),
        Some(auth) => auth
    };
    COLLECTION.with(|c|{
        let mut c = c.borrow_mut();
        let collection = Collection{
            name: arg.name,
            symbol: arg.symbol,
            royalties: arg.royalties,
            minting_authority: authority,
            royalty_recipient: arg.royalties_recipient,
            description: arg.description,
            image: arg.image,
            supply_cap: arg.supply_cap,
            ..Default::default()
        };
        *c = collection;
    })
}

// #[pre_upgrade]
// pub fn pre_upgrade(){
//     let collection = COLLECTION.with(|c| c.take());
//     ic_cdk::storage::stable_save((collection,)).expect("failed to save stable state");
// }

// #[post_upgrade]
// pub fn post_upgrade(){
//     let (collection,) = ic_cdk::storage::stable_restore().expect("failed to restore stable state");
//     COLLECTION.with(|c|{
//         *c.borrow_mut() = collection;
//     })
// }