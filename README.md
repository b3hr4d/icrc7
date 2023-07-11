# icrc7

## Features
- [x] Stable Memory
- [x] Pre and Post upgrading

<strong>Remaining/</strong>
- [ ] Transaction Deduplication
- [ ] Chunk Upload for larger images
- [ ] Transaction Log with filters
- [ ] HTTP Handling

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Build the project
chmod +x scripts/build.sh
./scripts/build.sh

# Starts the replica, running in the background
dfx start --background

# Sets minting authority
export MINTING_AUTHORITY=$(dfx identity get-principal)

# Deploys your canisters to the replica and generates your candid interface
dfx deploy icrc7 --argument '(record {
  name="Icrc7 Token";
  symbol="ICRC7";
  minting_authority=opt principal"'${MINTING_AUTHORITY}'";
  royalties=null;    
  royalties_recipient=null;    
  description=opt "ICRC7 Standard Token";
  image=null;    
  supply_cap=null;    
})'

# Mints token
dfx canister call icrc7 icrc7_mint '(record{
  id=100;
  name="Icrc7 100";
  description=opt "100th token of the collection";
  image=null;
  to=record{
  owner=principal"2vxsx-fae";
  subaccount=null;
  };
})'

# Approves token
dfx canister call icrc7 icrc7_approve '(record{
  from=null;
  to=principal"2vxsx-fae";
  tokenIds=null;
  expires_at=null;
  memo=null;
  created_at=null;
})'

# Transfers token
dfx canister call icrc7 icrc7_transfer '(record{
  from=null;
  to=record{
  owner=principal"r4bei-hre5h-74jey-tf5fd-j7pnu-tko5a-n6f6p-rpcgq-p7eov-q6gyk-vae";
  subaccount=null;
  };
  memo=null;
  created_at_time=null;
  is_atomic=null;token_ids=vec{100}; 
})'

# Returns owner of
dfx canister call icrc7_owner_of '(100)'
```