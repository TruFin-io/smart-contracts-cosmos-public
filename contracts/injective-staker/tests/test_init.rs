pub mod helpers;

#[cfg(test)]
mod staker_init {

    use cosmwasm_std::{to_json_binary, Addr, Attribute, Coin, Uint128, WasmMsg};
    use cw_multi_test::{Executor, IntoBech32};
    use helpers::{contract_wrapper, instantiate_staker, mock_app_with_validator};
    use injective_staker::{
        msg::{GetStakerInfoResponse, GetValidatorResponse, InstantiateMsg, QueryMsg},
        state::{ValidatorInfo, ValidatorState},
        INJ, ONE_INJ,
    };
    use injective_test_tube::{Account, InjectiveTestApp, Module, Wasm};

    use crate::helpers::{self, mint_inj};

    #[test]
    fn test_instantiate_staker() {
        let owner: Addr = "owner".into_bech32();
        let treasury: Addr = "treasury".into_bech32();

        let (app, staking_contract, default_validator) =
            instantiate_staker(owner.clone(), treasury.clone());

        let staker_info: GetStakerInfoResponse = app
            .wrap()
            .query_wasm_smart(staking_contract.clone(), &QueryMsg::GetStakerInfo {})
            .unwrap();

        assert_eq!(
            staker_info,
            GetStakerInfoResponse {
                default_validator: default_validator.to_string(),
                owner: owner.to_string(),
                treasury: treasury.to_string(),
                fee: 0,
                min_deposit: ONE_INJ.into(),
                is_paused: false,
            }
        );

        let response: GetValidatorResponse = app
            .wrap()
            .query_wasm_smart(staking_contract, &QueryMsg::GetValidators {})
            .unwrap();

        assert_eq!(
            response.validators,
            vec![ValidatorInfo {
                total_staked: Uint128::zero(),
                state: ValidatorState::Enabled,
                addr: default_validator.to_string(),
            }]
        );
    }

    #[test]
    fn test_emit_instantiate_event() {
        let owner: Addr = "owner".into_bech32();
        let treasury: Addr = "treasury".into_bech32();

        let (mut app, validator_addr) = mock_app_with_validator();
        let code_id = app.store_code(contract_wrapper());

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            treasury: treasury.to_string(),
            default_validator: validator_addr.to_string(),
        };

        mint_inj(&mut app, &owner, 1);
        let msg = WasmMsg::Instantiate {
            admin: Some(owner.to_string()),
            code_id,
            msg: to_json_binary(&msg).unwrap(),
            funds: [Coin::new(1u128, INJ)].to_vec(),
            label: "staker-contract".into(),
        };
        let init_response = app.execute(owner.clone(), msg.into());
        assert!(init_response.is_ok());

        let response = init_response.unwrap();
        let init_event = response.events.last().unwrap();
        assert_eq!(init_event.ty, "wasm-instantiated");

        assert_eq!(
            init_event.attributes.get(1).unwrap(),
            Attribute {
                key: "owner".to_string(),
                value: owner.to_string()
            }
        );
        assert_eq!(
            init_event.attributes.get(2).unwrap(),
            Attribute {
                key: "default_validator".to_string(),
                value: validator_addr.to_string()
            }
        );
        assert_eq!(
            init_event.attributes.get(3).unwrap(),
            Attribute {
                key: "treasury".to_string(),
                value: treasury.to_string()
            }
        );
    }

    #[test]
    fn test_instantiate_with_injective_tube() {
        let app = InjectiveTestApp::new();

        let accounts = app.init_accounts(&[Coin::new(ONE_INJ, "inj")], 2).unwrap();

        let admin = &accounts[0];
        let treasury = &accounts[1];
        let validator = app.get_first_validator_address().unwrap();
        let wasm = Wasm::new(&app);

        let wasm_byte_code = std::fs::read("./tests/test_artifacts/injective_staker.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, admin)
            .unwrap()
            .data
            .code_id;

        assert_eq!(code_id, 1u64);

        // instantiate the contract
        let response = wasm.instantiate(
            code_id,
            &InstantiateMsg {
                owner: admin.address(),
                treasury: treasury.address(),
                default_validator: validator,
            },
            None,
            Some("Test Staker"),
            &[Coin::new(1u128, INJ)],
            admin,
        );

        assert!(response.is_ok());

        let contract_addr = response.unwrap().data.address;
        assert!(contract_addr.starts_with("inj"));
    }
}
