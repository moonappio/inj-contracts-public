use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::App;

use crate::{
    helpers::MoonClaimContract,
    msg::UserInfoItem,
    state::{ClaimInfo, UserInfo},
};

const REWARD_DENOM: &str = "reward_denom";

#[test]
pub fn claim() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let claim_info = ClaimInfo {
        reward_denom: REWARD_DENOM.to_string(),
        initial_unlock: 2000,
        vesting_start: Timestamp::from_seconds(0),
        vesting_cliff: 30,
        vesting_time: 40,
        vesting_interval: 1,
    };

    let mut app = App::default();
    let code_id = MoonClaimContract::store_code(&mut app);

    let contract =
        MoonClaimContract::instantiate(&mut app, code_id, &owner, "Contract", claim_info).unwrap();

    MoonClaimContract::mint_coins(&mut app, &contract.0, coins(1000000, REWARD_DENOM));

    // test 1
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(40),
        chain_id: "random-test".to_string(),
    });

    let users = vec![UserInfoItem {
        address: user.clone().to_string(),
        user_info: UserInfo {
            reward: 1000,
            withdrawn: 0,
        },
    }];
    contract.set_users(&mut app, &owner, users.clone()).unwrap();

    contract.claim(&mut app, &user).unwrap();
    let res = app
        .wrap()
        .query_balance(user.clone(), REWARD_DENOM)
        .unwrap();
    assert_eq!(res, coin(400, REWARD_DENOM));

    // test 2
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(70),
        chain_id: "random-test".to_string(),
    });

    let users = vec![UserInfoItem {
        address: user.clone().to_string(),
        user_info: UserInfo {
            reward: 100,
            withdrawn: 0,
        },
    }];
    contract.set_users(&mut app, &owner, users).unwrap();

    contract.claim(&mut app, &user).unwrap();
    let res = app
        .wrap()
        .query_balance(user.clone(), REWARD_DENOM)
        .unwrap();
    assert_eq!(res, coin(500, REWARD_DENOM));
}

#[test]
pub fn withdraw() {
    let owner = Addr::unchecked("owner");
    let user = Addr::unchecked("user");

    let claim_info = ClaimInfo {
        reward_denom: REWARD_DENOM.to_string(),
        initial_unlock: 2000,
        vesting_start: Timestamp::from_seconds(0),
        vesting_cliff: 30,
        vesting_time: 40,
        vesting_interval: 1,
    };

    let mut app = App::default();
    let code_id = MoonClaimContract::store_code(&mut app);

    let contract =
        MoonClaimContract::instantiate(&mut app, code_id, &owner, "Contract", claim_info).unwrap();

    MoonClaimContract::mint_coins(&mut app, &contract.0, coins(1000000, REWARD_DENOM));

    contract
        .withdraw(
            &mut app,
            &owner,
            owner.to_string(),
            REWARD_DENOM.to_string(),
            900000,
        )
        .unwrap();
    let res = app
        .wrap()
        .query_balance(owner.clone(), REWARD_DENOM)
        .unwrap();
    assert_eq!(res, coin(900000, REWARD_DENOM));

    let res = app.wrap().query_balance(&contract.0, REWARD_DENOM).unwrap();
    assert_eq!(res, coin(100000, REWARD_DENOM));

    // test must be owner
    let res = contract.withdraw(
        &mut app,
        &user,
        user.to_string(),
        REWARD_DENOM.to_string(),
        100000,
    );

    assert!(res.is_err());
}
