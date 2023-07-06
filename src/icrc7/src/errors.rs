use candid::{Nat, CandidType, Principal};

#[derive(CandidType)]
pub enum TransferError{
    Unauthorized{ tokens_ids: Vec<Nat> },
    TooOld,
    CreatedInFuture{ ledger_time: u64 },
    Duplicate{ duplicate_of: Nat },
    TemporaryUnavailable,
    GenericError{ error_code: Nat, msg: String },
}

#[derive(CandidType)]
pub enum ApprovalError{
    Unauthorized{ tokens_ids: Vec<Nat> },
    TooOld,
    TemporaryUnavailable,
    GenericError{ error_code: Nat, msg: String },
}

#[derive(CandidType)]
pub enum MintError{
    SupplyCapReached,
    Unauthorized{ minting_authority: Principal },
}