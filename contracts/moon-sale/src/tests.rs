use cosmwasm_std::{
    coins, from_json,
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Timestamp,
};
use cw_multi_test::BankKeeper;

use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, GetSaleResponse, InstantiateMsg, QueryMsg, UserInfoReq, UserInfoRes},
    state::SaleConfig,
    ContractError,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(0),
        end_time: Timestamp::from_seconds(0),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };
    let msg = InstantiateMsg { sale: sale.clone() };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let sale_res = query(deps.as_ref(), mock_env(), QueryMsg::GetSale {}).unwrap();
    let sale_value: GetSaleResponse = from_json(sale_res).unwrap();
    assert_eq!(sale, sale_value.sale_config);
    assert_eq!(0, sale_value.sale_data.total_spent);
}

#[test]
pub fn update_sale() {
    let mut deps = mock_dependencies();
    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(0),
        end_time: Timestamp::from_seconds(0),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let instantiate_msg = InstantiateMsg { sale };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let new_sale = SaleConfig {
        start_time: Timestamp::from_seconds(1),
        end_time: Timestamp::from_seconds(1),
        pay_denom: "usdt2".to_string(),
        sale_denom: Some("moon2".to_string()),
        max_supply: 2000000000000000000000000000,
        price_pay_amount: 3000000000000000000,
        price_receive_amount: 3000000000000000000,
    };

    let msg = ExecuteMsg::UpdateSale {
        sale: new_sale.clone(),
    };

    // Prevent unauthorized update
    let unauth_info = mock_info("anyone", &[]);
    let err = execute(deps.as_mut(), mock_env(), unauth_info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    // Unauthroized - end

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let sale_res = query(deps.as_ref(), mock_env(), QueryMsg::GetSale {}).unwrap();
    let sale_value: GetSaleResponse = from_json(sale_res).unwrap();
    assert_eq!(new_sale, sale_value.sale_config);
}

#[test]
pub fn set_user_list() {
    let mut deps = mock_dependencies();
    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(0),
        end_time: Timestamp::from_seconds(0),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    let instantiate_msg = InstantiateMsg { sale };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let users = vec![
        UserInfoReq {
            address: "addr1".to_string(),
            allocation: 100,
            spent: 0,
        },
        UserInfoReq {
            address: "addr2".to_string(),
            allocation: 200,
            spent: 50,
        },
    ];

    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };

    // Prevent unauthorized update
    let unauth_info = mock_info("anyone", &[]);
    let err = execute(deps.as_mut(), mock_env(), unauth_info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // user: addr1
    let user_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUserInfo {
            address: "addr1".to_string(),
        },
    )
    .unwrap();
    let user_value: UserInfoRes = from_json(user_res).unwrap();
    assert_eq!(users[0].address, user_value.address);
    assert_eq!(users[0].allocation, user_value.allocation);
    assert_eq!(users[0].spent, user_value.spent);
    assert_eq!(0, user_value.received_amount);

    // user: addr2
    let user_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUserInfo {
            address: "addr2".to_string(),
        },
    )
    .unwrap();
    let user_value: UserInfoRes = from_json(user_res).unwrap();
    assert_eq!(users[1].address, user_value.address);
    assert_eq!(users[1].allocation, user_value.allocation);
    assert_eq!(users[1].spent, user_value.spent);
    assert_eq!(0, user_value.received_amount);

    // user: unexisting
    let user_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUserInfo {
            address: "unexisting".to_string(),
        },
    )
    .unwrap();
    let user_value: UserInfoRes = from_json(user_res).unwrap();
    assert_eq!(
        UserInfoRes {
            address: "unexisting".to_string(),
            allocation: 0,
            spent: 0,
            received_amount: 0,
        },
        user_value
    );

    // users()
    let users_res = query(deps.as_ref(), mock_env(), QueryMsg::GetUsers {}).unwrap();
    let users_value: Vec<UserInfoRes> = from_json(users_res).unwrap();
    assert_eq!(
        users
            .iter()
            .map(move |v| UserInfoRes {
                address: v.address.clone(),
                allocation: v.allocation,
                spent: v.spent,
                received_amount: 0,
            })
            .collect::<Vec<UserInfoRes>>(),
        users_value
    );
}

#[test]
pub fn buy_success() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1000, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 100,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(3, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let sale_res = query(deps.as_ref(), env.clone(), QueryMsg::GetSale {}).unwrap();
    let sale_value: GetSaleResponse = from_json(sale_res).unwrap();
    assert_eq!(3, sale_value.sale_data.total_spent);

    let user_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetUserInfo {
            address: user.to_string(),
        },
    );
    let user_value: UserInfoRes = from_json(user_res.unwrap()).unwrap();
    assert_eq!(
        UserInfoRes {
            address: user.to_string(),
            allocation: 100,
            spent: 3,
            received_amount: 3,
        },
        user_value
    );
}

#[test]
pub fn buy_success_with_different_price() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1000, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_receive_amount: 2,
        price_pay_amount: 1,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 100,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(4, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let sale_res = query(deps.as_ref(), env.clone(), QueryMsg::GetSale {}).unwrap();
    let sale_value: GetSaleResponse = from_json(sale_res).unwrap();
    assert_eq!(4, sale_value.sale_data.total_spent);

    let user_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetUserInfo {
            address: user.to_string(),
        },
    );
    let user_value: UserInfoRes = from_json(user_res.unwrap()).unwrap();
    assert_eq!(
        UserInfoRes {
            address: user.to_string(),
            allocation: 100,
            spent: 4,
            received_amount: 8,
        },
        user_value
    );
}

#[test]
pub fn buy_without_user_allocation() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 0,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(1000, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(Err(ContractError::NotParticipating {}), _res);
}

#[test]
pub fn buy_without_user_available_allocation() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 1000000000000000000000000000,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 100,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(1000, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(
        Err(ContractError::UserAllocationExceeded {
            wanted: 1000,
            max: 100
        }),
        _res
    );
}

#[test]
pub fn buy_without_sale_available_allocation() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 1000,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(1000, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(
        Err(ContractError::SaleAllocationExceeded {
            wanted: 1000,
            max: 10
        }),
        _res
    );
}

#[test]
pub fn buy_with_sale_not_active() {
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(0);

    BankKeeper::new()
        .init_balance(&mut deps.storage, &user, coins(1, "uusd"))
        .unwrap();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // set users
    let users = vec![UserInfoReq {
        address: user.to_string(),
        allocation: 1000,
        spent: 0,
    }];
    let msg = ExecuteMsg::SetUserList {
        users: users.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(user.as_str(), &coins(1000, "uusd"));
    let msg = ExecuteMsg::Buy {};
    let _res = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(Err(ContractError::SaleNotActive {}), _res);
}

#[test]
pub fn calculate_token_received_amount() {
    let owner = Addr::unchecked("owner");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let env = mock_env();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    let amount = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetReceivedAmount { pay_amount: 218 },
    )
    .unwrap();
    let amount_value: u128 = from_json(amount).unwrap();

    assert_eq!(218, amount_value);

    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::UpdateSale {
            sale: SaleConfig {
                start_time: Timestamp::from_seconds(3),
                end_time: Timestamp::from_seconds(10),
                pay_denom: "uusd".to_string(),
                sale_denom: Some("moon".to_string()),
                max_supply: 1000000000000000000000000000,
                price_pay_amount: 2,
                price_receive_amount: 129874,
            },
        },
    )
    .unwrap();

    let amount = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetReceivedAmount { pay_amount: 1321 },
    )
    .unwrap();
    let amount_value: u128 = from_json(amount).unwrap();
    assert_eq!(85781777, amount_value);

    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateSale {
            sale: SaleConfig {
                start_time: Timestamp::from_seconds(3),
                end_time: Timestamp::from_seconds(10),
                pay_denom: "uusd".to_string(),
                sale_denom: Some("moon".to_string()),
                max_supply: 1000000000000000000000000000,
                price_pay_amount: 3,
                price_receive_amount: 2,
            },
        },
    )
    .unwrap();

    let amount = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetReceivedAmount { pay_amount: 1000 },
    )
    .unwrap();
    let amount_value: u128 = from_json(amount).unwrap();
    assert_eq!(666, amount_value);
}

#[test]
pub fn change_ownership() {
    let owner = Addr::unchecked("owner");
    let any = Addr::unchecked("any");

    let mut deps = mock_dependencies();
    let info = mock_info(owner.as_str(), &[]);
    let env = mock_env();

    let sale: SaleConfig = SaleConfig {
        start_time: Timestamp::from_seconds(3),
        end_time: Timestamp::from_seconds(10),
        pay_denom: "uusd".to_string(),
        sale_denom: Some("moon".to_string()),
        max_supply: 10,
        price_pay_amount: 1000000000000000000,
        price_receive_amount: 1000000000000000000,
    };

    // instantiate
    let instantiate_msg = InstantiateMsg { sale };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::TransferOwnership {
            address: any.to_string(),
        },
    )
    .unwrap();

    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::TransferOwnership {
            address: any.to_string(),
        },
    );
    assert_eq!(Err(ContractError::Unauthorized {}), _res);

    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(any.as_str(), &[]),
        ExecuteMsg::TransferOwnership {
            address: owner.to_string(),
        },
    );
    assert!(_res.is_ok());
}
