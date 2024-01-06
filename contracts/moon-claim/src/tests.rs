use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, mock_info},
    Timestamp,
};

use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UserInfoItem},
    state::{ClaimInfo, UserInfo},
};

const REWARD_DENOM: &str = "reward_denom";
const NOW: Timestamp = Timestamp::from_seconds(1618308000);

fn default_msg() -> InstantiateMsg {
    InstantiateMsg {
        claim_info: ClaimInfo {
            reward_denom: REWARD_DENOM.to_string(),
            initial_unlock: 1000,
            vesting_start: NOW,
            vesting_cliff: 1,
            vesting_time: 1,
            vesting_interval: 1,
        },
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);

    // seccessfully instantiate
    let res = instantiate(deps.as_mut(), mock_env(), info, default_msg()).unwrap();
    assert_eq!(0, res.messages.len());

    // set claim info
    let claim_res = query(deps.as_ref(), mock_env(), QueryMsg::GetClaim {}).unwrap();
    let claim_res: ClaimInfo = from_json(claim_res).unwrap();
    assert_eq!(claim_res, default_msg().claim_info);

    // users are empty
    let user_res = query(deps.as_ref(), mock_env(), QueryMsg::GetUsers {}).unwrap();
    let user_res: Vec<UserInfoItem> = from_json(user_res).unwrap();
    assert_eq!(user_res.len(), 0);

    // fetches state
    let state_res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
    let state_res: crate::state::State = from_json(state_res).unwrap();
    assert!(!state_res.paused);
    assert_eq!(state_res.owner, "creator".to_string());
}

#[test]
fn get_set_claim_info() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), default_msg()).unwrap();

    let claim_info = ClaimInfo {
        reward_denom: REWARD_DENOM.to_string(),
        initial_unlock: 2000,
        vesting_start: NOW.plus_seconds(2),
        vesting_cliff: 2,
        vesting_time: 2,
        vesting_interval: 2,
    };

    let msg = ExecuteMsg::UpdateClaimInfo {
        claim_info: claim_info.clone(),
    };

    // Test for unauthorized
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[]),
        msg.clone(),
    );
    assert!(_res.is_err());

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let claim_res = query(deps.as_ref(), mock_env(), QueryMsg::GetClaim {}).unwrap();
    let claim_res: ClaimInfo = from_json(claim_res).unwrap();
    assert_eq!(claim_res, claim_info);
}

#[test]
fn get_set_users() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), default_msg()).unwrap();

    let user_res = query(deps.as_ref(), mock_env(), QueryMsg::GetUsers {}).unwrap();
    let user_res: Vec<UserInfoItem> = from_json(user_res).unwrap();
    assert_eq!(user_res.len(), 0);

    let users = vec![
        UserInfoItem {
            address: "addr1".to_string(),
            user_info: UserInfo {
                reward: 100,
                withdrawn: 10,
            },
        },
        UserInfoItem {
            address: "addr2".to_string(),
            user_info: UserInfo {
                reward: 200,
                withdrawn: 20,
            },
        },
    ];

    let msg = ExecuteMsg::SetUsers {
        users: users.clone(),
    };

    // Test for unauthorized
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[]),
        msg.clone(),
    );
    assert!(_res.is_err());

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let user_res = query(deps.as_ref(), mock_env(), QueryMsg::GetUsers {}).unwrap();
    let user_res: Vec<UserInfoItem> = from_json(user_res).unwrap();
    assert_eq!(user_res.len(), 2);

    assert_eq!(user_res[0].address, "addr1".to_string());
    assert_eq!(user_res[1].address, "addr2".to_string());
    assert_eq!(user_res[0].user_info, users[0].user_info);
    assert_eq!(user_res[1].user_info, users[1].user_info);

    let user_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUserInfo {
            address: "addr1".to_string(),
        },
    )
    .unwrap();
    let user_res: UserInfo = from_json(user_res).unwrap();
    assert_eq!(user_res.reward, 100);
    assert_eq!(user_res.withdrawn, 10);
}

#[test]
fn claim_unlocked_amount() {
    let claim_info = ClaimInfo {
        reward_denom: REWARD_DENOM.to_string(),
        initial_unlock: 2000,
        vesting_start: Timestamp::from_seconds(0),
        vesting_cliff: 30,
        vesting_time: 40,
        vesting_interval: 1,
    };

    assert_eq!(
        claim_info
            .unlocked_amount(100, Timestamp::from_seconds(0))
            .unwrap(),
        20
    );
    assert_eq!(
        claim_info
            .unlocked_amount(100, Timestamp::from_seconds(70))
            .unwrap(),
        100
    );
    assert_eq!(
        claim_info
            .unlocked_amount(100, Timestamp::from_seconds(31))
            .unwrap(),
        22
    );
    assert_eq!(
        claim_info
            .unlocked_amount(1000, Timestamp::from_seconds(40))
            .unwrap(),
        400
    );
}
