const test = require("tape");
const { Ed25519KeyIdentity } = require("@dfinity/identity");

const {
    idlFactory: factory_interface
} = require("./../.dfx/local/canisters/factory/factory.did.test.cjs");

const {
    idlFactory: icrc7_interace
} = require("./icrc7.did.test.cjs");

const { getActor } = require("./actor.cjs");

const canister_ids = require("./../.dfx/local/canister_ids.json");
const { Principal } = require("@dfinity/principal");
const { encodeIcrcAccount } = require("@dfinity/ledger");
const factory = canister_ids.factory.local;

let factory_actors = {}

let icrc7_canister = ""
let icrc7_actors = {}

// identities
let minter = Ed25519KeyIdentity.generate();

let user1 = Ed25519KeyIdentity.generate();
let user2 = Ed25519KeyIdentity.generate();
let user3 = Ed25519KeyIdentity.generate();
let user4 = Ed25519KeyIdentity.generate();
let user5 = Ed25519KeyIdentity.generate();

let subaccount = new Uint8Array(32).fill(0);

// accounts
let user2Account = encodeIcrcAccount({ owner: user2.getPrincipal(), subaccount: [subaccount] });
let user3Account = encodeIcrcAccount({ owner: user3.getPrincipal(), subaccount: [subaccount] });
let user4Account = encodeIcrcAccount({ owner: user4.getPrincipal(), subaccount: [subaccount] });

test("setup minting actor", async function (t) {
    console.log("=====factory=====");
    factory_actors.minter = await getActor(
        factory,
        factory_interface,
        minter
    )
})

test("Mint, should return an address", async function (t) {
    const response = await factory_actors.minter.create_icrc7_collection({
        'supply_cap': [10],
        'name': "Btc Flower",
        'description': [],
        'royalties': [],
        'image': [],
        'royalties_recipient': [],
        'symbol': "BTC"
    })
    console.log(response.toString())
    icrc7_canister = response
})

test("setting icrc7 actors", async function (t) {
    console.log("====icrc7 actors====");
    icrc7_actors.minter = await getActor(
        icrc7_canister,
        icrc7_interace,
        minter
    )
    icrc7_actors.user1 = await getActor(
        icrc7_canister,
        icrc7_interace,
        user1
    )
    icrc7_actors.user2 = await getActor(
        icrc7_canister,
        icrc7_interace,
        user2
    )
    icrc7_actors.user3 = await getActor(
        icrc7_canister,
        icrc7_interace,
        user3
    )
    icrc7_actors.user4 = await getActor(
        icrc7_canister,
        icrc7_interace,
        user4
    )
    icrc7_actors.user5 = await getActor(
        icrc7_canister,
        icrc7_interace,
        user5
    )
    // icrc7_actors.user6 = await getActor(
    //     icrc7_canister,
    //     icrc7_interace,
    //     user6
    // )
    // icrc7_actors.user7 = await getActor(
    //     icrc7_canister,
    //     icrc7_interace,
    //     user7
    // )
    // icrc7_actors.user8 = await getActor(
    //     icrc7_canister,
    //     icrc7_interace,
    //     user8
    // )
    // icrc7_actors.user9 = await getActor(
    //     icrc7_canister,
    //     icrc7_interace,
    //     user9
    // )
    // icrc7_actors.user10 = await getActor(
    //     icrc7_canister,
    //     icrc7_interace,
    //     user10
    // )
})

test("should return error about supply cap after 10 mints", async function (t) {
    for (i = 1n; i < 11; i++) {
        var id = await icrc7_actors.minter.icrc7_mint({
            'id': i,
            'to': {
                owner: user1.getPrincipal(),
                subaccount: []
            },
            'name': "Token",
            'description': [],
            'image': []
        })
        t.equal(id, i)
    };
    await icrc7_actors.minter.icrc7_mint({
        'id': 11n,
        'to': {
            owner: user1.getPrincipal(),
            subaccount: []
        },
        'name': "Token",
        'description': [],
        'image': []
    })
})

test("transfer and check owner", async function (t) {
    let response = await icrc7_actors.user1.icrc7_transfer({
        'to': {
            owner: user2.getPrincipal(),
            subaccount: []
        },
        'from': [],
        'memo': [],
        'is_atomic': [],
        'token_ids': [1n],
        'created_at_time': []
    })
    console.log(response)
    var owner = await icrc7_actors.user1.icrc7_owner_of(1n);
    t.equal(user2Account, encodeIcrcAccount(owner))
})

test("multiple token transfer", async function (t) {
    let { Ok: ok, Err: e } = await icrc7_actors.user1.icrc7_transfer({
        'to': {
            owner: user2.getPrincipal(),
            subaccount: []
        },
        'from': [],
        'memo': [],
        'is_atomic': [],
        'token_ids': [2n, 3n, 4n,],
        'created_at_time': []
    })
    console.log(ok);
    var owner = await icrc7_actors.user1.icrc7_owner_of(2n);
    var owner = encodeIcrcAccount(owner)
    t.equal(user2Account, owner)
    var owner = await icrc7_actors.user1.icrc7_owner_of(3n);
    var owner = encodeIcrcAccount(owner)
    t.equal(user2Account, owner)
    var owner = await icrc7_actors.user1.icrc7_owner_of(4n);
    var owner = encodeIcrcAccount(owner)
    t.equal(user2Account, owner)
})

test("approve", async function (t) {
    let result = await icrc7_actors.user2.icrc7_approve({
        to: user3.getPrincipal(),
        'tokenIds': [],
        'memo': [],
        'created_at': [],
        'from_subaccount': [],
        'expires_at': [],
    })
    await icrc7_actors.user3.icrc7_transfer({
        'to': {
            owner: user4.getPrincipal(),
            subaccount: []
        },
        'from': [],
        'memo': [],
        'is_atomic': [],
        'token_ids': [3n, 4n],
        'created_at_time': []
    });
    var owner = await icrc7_actors.user3.icrc7_owner_of(3n);
    var owner = encodeIcrcAccount(owner)
    t.equal(user4Account, owner)
    var owner = await icrc7_actors.user3.icrc7_owner_of(4n);
    var owner = encodeIcrcAccount(owner)
    t.equal(user4Account, owner)
})