use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::App;

use crate::{
    helpers::MoonSaleContract,
    msg::{GetSaleResponse, QueryMsg, UserInfoReq, UserInfoRes},
    state::SaleConfig,
    ContractError,
};

#[test]
pub fn buy_success() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &user, coins(1000, "uusd"))
            .unwrap();
    });
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(5),
        chain_id: "random-test".to_string(),
    });
    let code_id = MoonSaleContract::store_code(&mut app);

    let contract =
        MoonSaleContract::instantiate(&mut app, code_id, &owner, "Contract", sale).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 100,
        spent: 0,
    }];
    contract
        .set_user_list(&mut app, &owner, users.clone())
        .unwrap();

    contract.buy(&mut app, &user, coins(3, "uusd")).unwrap();

    let sale_res: GetSaleResponse = contract.query_value(&app, &QueryMsg::GetSale {}).unwrap();
    assert_eq!(3, sale_res.sale_data.total_spent);

    let user_res: UserInfoRes = contract
        .query_value(
            &app,
            &QueryMsg::GetUserInfo {
                address: user.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        UserInfoRes {
            address: user.to_string(),
            allocation: 100,
            spent: 3,
            received_amount: 3,
        },
        user_res
    );
}

#[test]
pub fn buy_without_sale_available_allocation() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &user, coins(1000, "uusd"))
            .unwrap();
    });
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(5),
        chain_id: "random-test".to_string(),
    });
    let code_id = MoonSaleContract::store_code(&mut app);

    let contract =
        MoonSaleContract::instantiate(&mut app, code_id, &owner, "Contract", sale).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 1000,
        spent: 0,
    }];
    contract
        .set_user_list(&mut app, &owner, users.clone())
        .unwrap();

    let res = contract.buy(&mut app, &user, coins(1000, "uusd"));

    assert_eq!(
        Err(ContractError::SaleAllocationExceeded {
            wanted: 1000,
            max: 10
        }),
        res
    );
}

#[test]
pub fn buy_without_user_available_allocation() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &user, coins(1000, "uusd"))
            .unwrap();
    });
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(5),
        chain_id: "random-test".to_string(),
    });
    let code_id = MoonSaleContract::store_code(&mut app);

    let contract =
        MoonSaleContract::instantiate(&mut app, code_id, &owner, "Contract", sale).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 100,
        spent: 0,
    }];
    contract
        .set_user_list(&mut app, &owner, users.clone())
        .unwrap();

    let res = contract.buy(&mut app, &user, coins(1000, "uusd"));
    assert_eq!(
        Err(ContractError::UserAllocationExceeded {
            wanted: 1000,
            max: 100
        }),
        res
    );
}

#[test]
pub fn withdraw_success() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let mut app = App::default();
    let code_id = MoonSaleContract::store_code(&mut app);

    let contract =
        MoonSaleContract::instantiate(&mut app, code_id, &owner, "Contract", sale).unwrap();

    MoonSaleContract::mint_coins(&mut app, &contract.addr(), coins(100, "uusd"));

    contract
        .withdraw(&mut app, &owner, &user, coins(100, "uusd"))
        .unwrap();

    let res = app.wrap().query_balance(user, "uusd").unwrap();
    assert_eq!(res, coin(100, "uusd"));

    let res = app.wrap().query_balance(contract.addr(), "uusd").unwrap();
    assert_eq!(res, coin(0, "uusd"));
}

#[test]
pub fn withdraw_fail_insufficient_balance() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let mut app = App::default();
    let code_id = MoonSaleContract::store_code(&mut app);

    let contract =
        MoonSaleContract::instantiate(&mut app, code_id, &owner, "Contract", sale).unwrap();

    MoonSaleContract::mint_coins(&mut app, &contract.addr(), coins(100, "uusd"));

    let res = contract.withdraw(&mut app, &owner, &user, coins(200, "uusd"));

    assert!(res.is_err());
}
