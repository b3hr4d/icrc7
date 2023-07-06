use std::collections::HashSet;

use candid::candid_method;
use ic_cdk_macros::update;

use crate::{types::{TransferArgs, ApprovalArgs, MintArgs}, errors::{TransferError, ApprovalError, MintError}, state::{COLLECTION, Token}};

#[update]
#[candid_method(update)]
pub fn icrc7_transfer(arg: TransferArgs) -> Result<u128, TransferError>{
    let caller = ic_cdk::caller();
    COLLECTION.with(|c|{
        let mut c = c.borrow_mut();
        c.transfer(&caller, arg)
    })
}

#[update]
#[candid_method(update)]
pub fn icrc7_approve(arg: ApprovalArgs) -> Result<u128, ApprovalError>{
    let caller = ic_cdk::caller();
    COLLECTION.with(|c|{
        let mut c = c.borrow_mut();
        c.approve(&caller, arg)
    })
}

#[update]
#[candid_method(update)]
pub fn icrc7_mint(arg: MintArgs) -> Result<u128, MintError>{
    let caller = ic_cdk::caller();
    COLLECTION.with(|c|{
        let mut c = c.borrow_mut();
        let token = Token{
            id: arg.id,
            name: arg.name,
            description: arg.description,
            image: arg.image,
            owner: arg.to,
            approvals: HashSet::new()
        };
        c.mint(&caller, token)
    })
}