pub mod helpers;

#[cfg(test)]
mod setters {

    use crate::helpers::{
        self, add_validator, assert_error, assert_event_with_attributes, disable_validator,
    };

    use cosmwasm_std::{to_json_binary, Addr, Attribute, Uint128, WasmMsg};
    use cw_multi_test::{Executor, IntoBech32};
    use helpers::instantiate_staker;
    use injective_staker::{
        msg::{ExecuteMsg, GetStakerInfoResponse, QueryMsg},
        ONE_INJ,
    };

    #[test]
    fn test_set_fee() {
        let owner = "owner".into_bech32();
        let new_fee: u16 = 1000;

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetFee { new_fee }).unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert!(response.is_ok());

        let unwrapped_response = response.as_ref().unwrap();
        let execute_event = unwrapped_response.events.last().unwrap();

        // check to see the correct events are sent.
        assert_eq!(execute_event.ty, "wasm-set_fee");

        assert_eq!(
            execute_event.attributes.get(1).unwrap(),
            Attribute {
                key: "old_fee".to_string(),
                value: "0".to_string()
            }
        );

        assert_eq!(
            execute_event.attributes.get(2).unwrap(),
            Attribute {
                key: "new_fee".to_string(),
                value: new_fee.to_string()
            }
        );

        let staker_info: GetStakerInfoResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetStakerInfo {})
            .unwrap();

        assert_eq!(staker_info.fee, new_fee);
    }

    #[test]
    fn test_set_fee_entered_fee_above_precision() {
        let owner = "owner".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetFee { new_fee: 10500 }).unwrap(),
            funds: vec![],
        };

        let error_response = app.execute(owner, msg.into()).unwrap_err();
        let error_source = error_response.source().unwrap();

        assert_eq!(
            error_source.to_string(),
            "Fee cannot be larger than fee precision"
        );
    }

    #[test]
    fn test_set_fee_can_only_be_called_by_the_owner() {
        let owner = "owner".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner, "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetFee { new_fee: 1000 }).unwrap(),
            funds: vec![],
        };

        let error_response = app.execute("user".into_bech32(), msg.into()).unwrap_err();
        let error_source = error_response.source().unwrap();

        assert_eq!(
            error_source.to_string(),
            "Only the owner can call this method"
        );
    }

    #[test]
    fn test_set_min_deposit() {
        let owner = "owner".into_bech32();
        let new_min_deposit: u128 = 2 * ONE_INJ;

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());
        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetMinimumDeposit {
                new_min_deposit: new_min_deposit.into(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());

        assert!(response.is_ok());

        let response_event = response.as_ref().unwrap().events.last().unwrap();

        assert_eq!(response_event.ty, "wasm-set_min_deposit");

        assert_eq!(
            response_event.attributes.get(1).unwrap(),
            Attribute {
                key: "old_min_deposit".to_string(),
                value: ONE_INJ.to_string()
            }
        );

        assert_eq!(
            response_event.attributes.get(2).unwrap(),
            Attribute {
                key: "new_min_deposit".to_string(),
                value: new_min_deposit.to_string()
            }
        );

        let staker_info: GetStakerInfoResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetStakerInfo {})
            .unwrap();

        assert_eq!(staker_info.min_deposit, Uint128::from(new_min_deposit));
    }

    #[test]
    fn test_set_min_deposit_can_only_be_called_by_the_owner() {
        let owner = "owner".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner, "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetMinimumDeposit {
                new_min_deposit: ONE_INJ.into(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute("user".into_bech32(), msg.into());
        assert!(response.is_err());

        let error_response = response.as_ref().unwrap_err();
        let error_source = error_response.source().unwrap();

        assert_eq!(
            error_source.to_string(),
            "Only the owner can call this method"
        );
    }

    #[test]
    fn test_set_min_deposit_entered_deposit_less_than_min_deposit() {
        let owner = "owner".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetMinimumDeposit {
                new_min_deposit: Uint128::new(100_000_000),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert!(response.is_err());

        let error_response = response.as_ref().unwrap_err().source().unwrap();

        assert_eq!(
            error_response.to_string(),
            "Minimum deposit amount is too small"
        );
    }

    #[test]
    fn test_set_treasury() {
        let owner = "owner".into_bech32();
        let old_treasury_addr = "treasury".into_bech32();
        let new_treasury_addr = "new_treasury_addr".into_bech32();

        let (mut app, staker_addr, _) =
            instantiate_staker(owner.clone(), old_treasury_addr.clone());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetTreasury {
                new_treasury_addr: new_treasury_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert!(response.is_ok());

        assert_event_with_attributes(
            &response.unwrap().events,
            "wasm-set_treasury",
            vec![
                ("new_treasury_addr", new_treasury_addr.clone()).into(),
                ("old_treasury_addr", old_treasury_addr).into(),
            ],
            staker_addr.clone(),
        );

        let staker_info: GetStakerInfoResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetStakerInfo {})
            .unwrap();

        assert_eq!(staker_info.treasury, new_treasury_addr.to_string());
    }

    #[test]
    #[should_panic]
    fn test_set_treasury_to_wrong_address_fails() {
        let owner = "owner".into_bech32();
        let old_treasury_addr = "treasury".into_bech32();
        let new_treasury_addr = Addr::unchecked("new_treasury_addr");

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), old_treasury_addr);

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetTreasury {
                new_treasury_addr: new_treasury_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        app.execute(owner, msg.into()).unwrap();
    }

    #[test]
    fn test_set_treasury_not_called_by_owner_fails() {
        let owner = "owner".into_bech32();
        let old_treasury_addr = "treasury".into_bech32();
        let new_treasury_addr = "new_treasury_addr".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner, old_treasury_addr);

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetTreasury {
                new_treasury_addr: new_treasury_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(new_treasury_addr, msg.into());
        assert_error(response, "Only the owner can call this method");
    }

    #[test]
    fn test_set_default_validator() {
        let owner = "owner".into_bech32();
        let new_default_validator_addr = "new_default_validator_addr".into_bech32();

        let (mut app, staker_addr, old_default_validator_addr) =
            instantiate_staker(owner.clone(), "treasury".into_bech32());

        let add_validator_response = add_validator(
            &mut app,
            owner.clone(),
            &staker_addr,
            new_default_validator_addr.clone(),
        );
        assert!(add_validator_response.is_ok());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetDefaultValidator {
                new_default_validator_addr: new_default_validator_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert!(response.is_ok());

        assert_event_with_attributes(
            &response.unwrap().events,
            "wasm-set_default_validator",
            vec![
                (
                    "new_default_validator_addr",
                    new_default_validator_addr.clone(),
                )
                    .into(),
                ("old_default_validator_addr", old_default_validator_addr).into(),
            ],
            staker_addr.clone(),
        );

        let staker_info: GetStakerInfoResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetStakerInfo {})
            .unwrap();

        assert_eq!(
            staker_info.default_validator,
            new_default_validator_addr.to_string()
        );
    }

    #[test]
    fn test_set_default_validator_with_non_existent_validator_fails() {
        let owner = "owner".into_bech32();
        let new_default_validator_addr = "new_default_validator_addr".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetDefaultValidator {
                new_default_validator_addr: new_default_validator_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert_error(response, "Validator does not exist");
    }

    #[test]
    fn test_set_default_validator_with_disabled_validator_fails() {
        let owner = "owner".into_bech32();
        let new_default_validator_addr = "new_default_validator_addr".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner.clone(), "treasury".into_bech32());

        let add_validator_response = add_validator(
            &mut app,
            owner.clone(),
            &staker_addr,
            new_default_validator_addr.clone(),
        );
        assert!(add_validator_response.is_ok());

        disable_validator(
            &mut app,
            owner.clone(),
            &staker_addr,
            new_default_validator_addr.clone(),
        )
        .unwrap();
        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetDefaultValidator {
                new_default_validator_addr: new_default_validator_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(owner, msg.into());
        assert_error(response, "Validator is disabled");
    }

    #[test]
    fn test_set_default_validator_not_called_by_owner_fails() {
        let owner = "owner".into_bech32();
        let new_default_validator_addr = "new_default_validator_addr".into_bech32();

        let (mut app, staker_addr, _) = instantiate_staker(owner, "treasury".into_bech32());

        let msg = WasmMsg::Execute {
            contract_addr: staker_addr.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetDefaultValidator {
                new_default_validator_addr: new_default_validator_addr.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let response = app.execute(new_default_validator_addr, msg.into());
        assert_error(response, "Only the owner can call this method");
    }
}
