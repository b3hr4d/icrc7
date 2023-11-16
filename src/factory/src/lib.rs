use b3_utils::ledger::ICRCAccount;
use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use ic_cdk::{
    api::management_canister::main::{
        CanisterIdRecord, CanisterInstallMode, CanisterSettings, CreateCanisterArgument,
        InstallCodeArgument,
    },
    update,
};

#[derive(CandidType, Deserialize)]
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

#[derive(CandidType, Deserialize)]
pub struct CreateArg {
    pub name: String,
    pub symbol: String,
    pub royalties: Option<u16>,
    pub royalties_recipient: Option<ICRCAccount>,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub supply_cap: Option<u128>,
}

impl From<(Principal, CreateArg)> for Config {
    fn from((minting_authority, arg): (Principal, CreateArg)) -> Self {
        Config {
            name: arg.name,
            symbol: arg.symbol,
            minting_authority,
            royalties: arg.royalties,
            royalty_recipient: arg.royalties_recipient,
            description: arg.description,
            logo: arg.logo,
            supply_cap: arg.supply_cap,
            tx_window: 0,
            permitted_drift: 0,
        }
    }
}

const WASM: &[u8] =
    std::include_bytes!("./../../../target/wasm32-unknown-unknown/release/icrc7.wasm");

pub async fn get_an_address(caller: &Principal) -> Principal {
    ic_cdk::println!("{}", caller.clone());
    let canister_setting = CanisterSettings {
        controllers: Some(vec![caller.clone(), ic_cdk::id()]),
        compute_allocation: Some(Nat::from(0_u64)),
        memory_allocation: Some(Nat::from(0_u64)),
        freezing_threshold: Some(Nat::from(0_u64)),
    };
    let args = CreateCanisterArgument {
        settings: Some(canister_setting),
    };
    let (canister_id,): (CanisterIdRecord,) = match ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "create_canister",
        (args,),
        200_000_000_000,
    )
    .await
    {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord {
            canister_id: candid::Principal::anonymous(),
        },),
    };
    canister_id.canister_id
}

pub async fn install_wasm(wasm: Vec<u8>, canister_id: Principal, args: Vec<u8>) -> bool {
    let install_config = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        wasm_module: wasm,
        canister_id,
        arg: args,
    };
    match ic_cdk::api::call::call(
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    {
        Ok(x) => x,
        Err((rejection_code, msg)) => {
            ic_cdk::println!("{:?} {:?}", rejection_code, msg);
            return false;
        }
    }
    true
}

#[update]
pub async fn create_icrc7_collection(arg: CreateArg) -> Principal {
    let caller = ic_cdk::caller();
    let arg = Config::from((caller, arg));
    let address = get_an_address(&caller).await;
    if address == Principal::anonymous() {
        ic_cdk::trap("Failed to get an address")
    }
    let arg = Encode!(&arg).unwrap();
    match install_wasm(WASM.to_vec(), address, arg).await {
        true => address,
        false => ic_cdk::trap("Failed to install code"),
    }
}

ic_cdk::export_candid!();
