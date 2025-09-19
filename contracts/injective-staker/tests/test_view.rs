pub mod helpers;

#[cfg(test)]
mod view {

    use cosmwasm_std::{Addr, Decimal, Uint256};
    use cw_multi_test::{IntoBech32, StakingSudo};
    use helpers::{mint_inj, stake};
    use injective_staker::constants::INJ;
    use injective_staker::msg::{GetMaxWithdrawResponse, GetTotalAssetsResponse};
    use injective_staker::ONE_INJ;
    use injective_staker::{
        msg::{GetSharePriceResponse, GetTotalSupplyResponse, QueryMsg},
        SHARE_PRICE_SCALING_FACTOR,
    };

    use crate::helpers::{
        self, add_validator, claimable_amount, get_delegation, get_max_withdraw, get_share_price,
        get_total_rewards, get_total_staked, instantiate_staker,
        instantiate_staker_with_min_deposit, instantiate_staker_with_min_deposit_and_initial_stake,
        move_days_forward, unstake, unstake_when_rewards_accrue, whitelist_user,
    };

    #[test]
    fn test_get_total_supply() {
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, _) =
            instantiate_staker(owner.clone(), "treasury".into_bech32());

        let anyone: Addr = "anyone".into_bech32();

        let pre_total_supply: GetTotalSupplyResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalSupply {})
            .unwrap();
        assert!(pre_total_supply.total_supply.is_zero());

        // mint INJ tokens to the 'anyone' user
        let inj_to_mint = 10000000000000000000; // 10 INJ
        mint_inj(&mut app, &anyone, inj_to_mint);

        // whitelist user
        whitelist_user(&mut app, &contract_addr, &owner, &anyone);

        // execute stake
        stake(&mut app, &anyone, &contract_addr, inj_to_mint).unwrap();

        // ensure total supply was updated
        let post_total_supply: GetTotalSupplyResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::GetTotalSupply {})
            .unwrap();
        assert!(post_total_supply.total_supply.u128() == inj_to_mint);
    }

    #[test]
    fn test_get_total_staked_without_staking_multi_validator() {
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, _) =
            instantiate_staker(owner.clone(), "treasury".into_bech32());

        // add a second validator:
        let validator = "validator".into_bech32();
        add_validator(&mut app, owner, &contract_addr, validator).unwrap();

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.is_zero());
        assert!(total_rewards.is_zero());
    }

    #[test]
    fn test_get_total_staked_without_staking() {
        let owner = "owner".into_bech32();
        let (app, contract_addr, _) = instantiate_staker(owner, "treasury".into_bech32());

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.is_zero());
        assert!(total_rewards.is_zero());
    }

    #[test]
    fn test_get_total_staked_with_staking() {
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, _) =
            instantiate_staker(owner.clone(), "treasury".into_bech32());

        // mint INJ tokens to the 'anyone' user
        let anyone: Addr = "anyone".into_bech32();
        let inj_to_mint = 10000000000000000000; // 10 INJ
        mint_inj(&mut app, &anyone, inj_to_mint);

        // whitelist user
        whitelist_user(&mut app, &contract_addr, &owner, &anyone);

        // execute stake
        stake(&mut app, &anyone, &contract_addr, inj_to_mint).unwrap();

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards.is_zero());
    }

    #[test]
    fn test_get_total_staked_with_multi_validators() {
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, _) =
            instantiate_staker(owner.clone(), "treasury".into_bech32());

        // add a second validator:
        let validator = "validator".into_bech32();
        add_validator(&mut app, owner.clone(), &contract_addr, validator).unwrap();

        let anyone: Addr = "anyone".into_bech32();
        // mint INJ tokens to the 'anyone' user
        let inj_to_mint = 10000000000000000000; // 10 INJ
        mint_inj(&mut app, &anyone, inj_to_mint);

        // whitelist user
        whitelist_user(&mut app, &contract_addr, &owner, &anyone);

        // execute stake
        stake(&mut app, &anyone, &contract_addr, inj_to_mint).unwrap();

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards.is_zero());
    }

    #[test]
    fn test_get_total_rewards_with_rewards_accruing() {
        // instantiate the contract
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, validator_addr) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100000);

        let anyone: Addr = "anyone".into_bech32();
        // mint INJ tokens to the 'anyone' user
        let inj_to_mint = 1000000000000; // low enough INJ amount to not cause overflow error.
        mint_inj(&mut app, &anyone, inj_to_mint);

        // whitelist user
        whitelist_user(&mut app, &contract_addr, &owner, &anyone);

        // execute stake
        stake(&mut app, &anyone, &contract_addr, inj_to_mint).unwrap();

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards.is_zero());

        // simulate passage of time and reward accrual
        move_days_forward(&mut app, 1);

        // query total staked and rewards after reward distribution
        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        let delegation = get_delegation(&app, contract_addr.to_string(), &validator_addr);
        let acc_rewards = delegation
            .accumulated_rewards
            .iter()
            .find(|coin| coin.denom == INJ)
            .expect("INJ rewards not found");

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards == acc_rewards.amount);
    }

    #[test]
    fn test_get_total_rewards_with_multiple_validators() {
        let owner = "owner".into_bech32();
        let (mut app, contract_addr, validator_addr) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100000);

        // add a second validator:
        let validator = "validator".into_bech32();
        add_validator(&mut app, owner.clone(), &contract_addr, validator).unwrap();

        let anyone: Addr = "anyone".into_bech32();
        // mint INJ tokens to the 'anyone' user
        let inj_to_mint = 1000000000000; // low enough INJ amount to not cause overflow error.
        mint_inj(&mut app, &anyone, inj_to_mint);

        // whitelist user
        whitelist_user(&mut app, &contract_addr, &owner, &anyone);

        // execute stake
        stake(&mut app, &anyone, &contract_addr, inj_to_mint).unwrap();

        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards.is_zero());

        // simulate passage of time and reward accrual
        move_days_forward(&mut app, 1);

        // query total staked and rewards after reward distribution
        let total_staked = get_total_staked(&app, &contract_addr);
        let total_rewards = get_total_rewards(&app, &contract_addr);

        // fetch total rewards from validator
        let delegation = get_delegation(&app, contract_addr.to_string(), &validator_addr);
        let acc_rewards = delegation
            .accumulated_rewards
            .iter()
            .find(|coin| coin.denom == INJ)
            .expect("INJ rewards not found");

        assert!(total_staked.u128() == inj_to_mint);
        assert!(total_rewards == acc_rewards.amount);
    }

    #[test]
    fn test_get_total_assets() {
        let (mut app, staker_addr, _) =
            instantiate_staker("owner".into_bech32(), "treasury".into_bech32());

        // mint INJ tokens to the staker contract
        let staker_assets = 1234 * ONE_INJ;
        mint_inj(&mut app, &staker_addr, staker_assets);

        let response: GetTotalAssetsResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetTotalAssets {})
            .unwrap();

        assert_eq!(response.total_assets.u128(), staker_assets + 1); // +1 for reserve amount
    }

    #[test]
    fn test_get_share_price_when_no_shares_exist() {
        let (app, staker_addr, _) =
            instantiate_staker("owner".into_bech32(), "treasury".into_bech32());

        let response: GetSharePriceResponse = app
            .wrap()
            .query_wasm_smart(staker_addr, &QueryMsg::GetSharePrice {})
            .unwrap();

        assert_eq!(
            response.numerator,
            Uint256::from(SHARE_PRICE_SCALING_FACTOR)
        );
        assert_eq!(response.denominator, Uint256::from(1u64));
    }

    #[test]
    fn test_get_share_price_increases_with_rewards() {
        let owner = "owner".into_bech32();

        let (mut app, staker_addr, _) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100000);

        // stake some INJ
        let alice: Addr = "alice".into_bech32();
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        let stake_amount = 1000000000000;
        mint_inj(&mut app, &alice, stake_amount);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // verify initial share price
        let share_price_day_0 = get_share_price(&app, &staker_addr);
        assert_eq!(share_price_day_0, SHARE_PRICE_SCALING_FACTOR);

        // accrue rewards and verify share price increases
        move_days_forward(&mut app, 1);
        let share_price_day_1 = get_share_price(&app, &staker_addr);
        assert!(share_price_day_1 > share_price_day_0);

        // accrue rewards and verify share price increases
        move_days_forward(&mut app, 1);
        let share_price_day_2 = get_share_price(&app, &staker_addr);
        assert!(share_price_day_2 > share_price_day_1);
    }

    #[test]
    fn test_get_share_price_increases_with_multiple_validators() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100000);

        // add a second validator
        let second_validator = "second-validator".into_bech32();
        add_validator(&mut app, owner.clone(), &staker_addr, second_validator).unwrap();

        // stake some INJ
        let alice: Addr = "alice".into_bech32();
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        let stake_amount = 1000000000000;
        mint_inj(&mut app, &alice, stake_amount);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // verify initial share price
        let share_price_day_0 = get_share_price(&app, &staker_addr);
        assert_eq!(share_price_day_0, SHARE_PRICE_SCALING_FACTOR);

        // accrue rewards and verify share price increases
        move_days_forward(&mut app, 1);
        let share_price_day_1 = get_share_price(&app, &staker_addr);
        assert!(share_price_day_1 > share_price_day_0);

        // accrue rewards and verify share price increases
        move_days_forward(&mut app, 1);
        let share_price_day_2 = get_share_price(&app, &staker_addr);
        assert!(share_price_day_2 > share_price_day_1);
    }

    #[test]
    fn test_get_share_price_with_slashing() {
        let owner = "owner".into_bech32();
        let (mut app, staker_contract, validator_addr) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100000);

        // stake some INJ
        let alice: Addr = "alice".into_bech32();
        whitelist_user(&mut app, &staker_contract, &owner, &alice);
        let stake_amount = 1000000000000;
        mint_inj(&mut app, &alice, stake_amount);
        stake(&mut app, &alice, &staker_contract, stake_amount).unwrap();

        // verify initial share price
        let share_price_day_0 = get_share_price(&app, &staker_contract);
        assert_eq!(share_price_day_0, SHARE_PRICE_SCALING_FACTOR);

        // Slash the validator by 50%
        app.sudo(cw_multi_test::SudoMsg::Staking(StakingSudo::Slash {
            validator: validator_addr.to_string(),
            percentage: Decimal::percent(50),
        }))
        .unwrap();

        // verify share price decreases
        move_days_forward(&mut app, 1);
        let share_price_day_1 = get_share_price(&app, &staker_contract);
        assert!(share_price_day_1 < share_price_day_0);
    }

    #[test]
    fn test_max_withdraw_with_no_deposits() {
        let owner = "owner".into_bech32();
        let (app, staker_addr, _) = instantiate_staker(owner, "treasury".into_bech32());

        let alice = "alice".into_bech32();
        let response: GetMaxWithdrawResponse = app
            .wrap()
            .query_wasm_smart(
                staker_addr,
                &QueryMsg::GetMaxWithdraw {
                    user: alice.to_string(),
                },
            )
            .unwrap();

        assert_eq!(response.max_withdraw.u128(), 0);
    }

    #[test]
    fn test_max_withdraw_matches_deposits_when_no_rewards() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        let alice = "alice".into_bech32();
        mint_inj(&mut app, &alice, 100_000);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);

        // alice stakes some inj
        let first_stake = 70_000;
        stake(&mut app, &alice, &staker_addr, first_stake).unwrap();

        let response: GetMaxWithdrawResponse = app
            .wrap()
            .query_wasm_smart(
                staker_addr.clone(),
                &QueryMsg::GetMaxWithdraw {
                    user: alice.to_string(),
                },
            )
            .unwrap();

        assert_eq!(response.max_withdraw.u128(), first_stake);

        // alice stakes more inj
        let second_stake = 30_000;
        stake(&mut app, &alice, &staker_addr, second_stake).unwrap();

        // verify max withdraw is the sum of the two stakes
        let response: GetMaxWithdrawResponse = app
            .wrap()
            .query_wasm_smart(
                staker_addr.clone(),
                &QueryMsg::GetMaxWithdraw {
                    user: alice.to_string(),
                },
            )
            .unwrap();

        assert_eq!(response.max_withdraw.u128(), first_stake + second_stake);
    }

    #[test]
    fn test_max_withdraw_is_greater_than_deposits_when_rewards_accrue() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100_000);

        // alice stakes some INJ
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 1);

        // verify that max withdraw is greater than the initial stake amount
        let response: GetMaxWithdrawResponse = app
            .wrap()
            .query_wasm_smart(
                staker_addr.clone(),
                &QueryMsg::GetMaxWithdraw {
                    user: alice.to_string(),
                },
            )
            .unwrap();

        assert!(response.max_withdraw.u128() > stake_amount);
    }

    #[test]
    fn test_max_withdraw_increases_with_rewards() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 100_000);

        // alice stakes some INJ
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // verify that max withdraw matches the initial stake amount
        let first = get_max_withdraw(&app, &staker_addr, &alice);
        assert_eq!(first, stake_amount);
        // rewards accrue
        move_days_forward(&mut app, 1);

        // verify that max withdraw increased with rewards
        let second = get_max_withdraw(&app, &staker_addr, &alice);
        assert!(second > first);
        // rewards accrue
        move_days_forward(&mut app, 1);

        // verify that max withdraw increased with rewards
        let third = get_max_withdraw(&app, &staker_addr, &alice);
        assert!(third > second);
    }

    #[test]
    fn test_max_withdraw_is_zero_when_all_stake_is_unstaked() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 30);

        // get max withdraw
        let max_withdraw = get_max_withdraw(&app, &staker_addr, &alice);
        println!("max_withdraw: {}", max_withdraw);

        // unstake all
        let _ = unstake(&mut app, &alice, &staker_addr, max_withdraw);

        // verify max withdraw is zero
        let max_withdraw = get_max_withdraw(&app, &staker_addr, &alice);
        assert_eq!(max_withdraw, 0);
    }

    #[test]
    fn test_is_claimable_is_0_when_no_claims_are_claimable() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 30);

        // unstake
        unstake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // time passes but not 21 days
        move_days_forward(&mut app, 20);

        let claimable_amount = claimable_amount(&app, &alice, &staker_addr);
        assert!(claimable_amount.is_zero());
    }

    #[test]
    fn test_is_claimable_is_0_when_no_claims_exist() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        let claimable_amount = claimable_amount(&app, &alice, &staker_addr);
        assert!(claimable_amount.is_zero());
    }

    #[test]
    fn test_is_claimable() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 30);

        // unstake
        unstake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // 21 days pass
        move_days_forward(&mut app, 21);

        let claimable_amount = claimable_amount(&app, &alice, &staker_addr);
        assert!(claimable_amount.u128() == stake_amount);
    }

    #[test]
    fn test_is_claimable_several_claims() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 30);

        // unstake fully over multiple unstakes
        unstake(&mut app, &alice, &staker_addr, 20000).unwrap();

        move_days_forward(&mut app, 10);
        unstake(&mut app, &alice, &staker_addr, 50000).unwrap();
        unstake(&mut app, &alice, &staker_addr, 15000).unwrap();

        move_days_forward(&mut app, 1);
        unstake(&mut app, &alice, &staker_addr, 15000).unwrap();

        move_days_forward(&mut app, 21);

        let claimable_amount = claimable_amount(&app, &alice, &staker_addr);
        assert!(claimable_amount.u128() == stake_amount);
    }

    #[test]
    fn test_is_claimable_several_claims_only_some_claimable() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, _) = instantiate_staker_with_min_deposit_and_initial_stake(
            owner.clone(),
            "treasury".into_bech32(),
            0,
            100_000,
        );

        // alice stakes some inj
        let alice = "alice".into_bech32();
        let stake_amount = 100_000;
        mint_inj(&mut app, &alice, stake_amount);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);
        stake(&mut app, &alice, &staker_addr, stake_amount).unwrap();

        // rewards accrue
        move_days_forward(&mut app, 30);

        unstake(&mut app, &alice, &staker_addr, 20000).unwrap();

        move_days_forward(&mut app, 20);
        unstake(&mut app, &alice, &staker_addr, 50000).unwrap();
        unstake(&mut app, &alice, &staker_addr, 15000).unwrap();

        move_days_forward(&mut app, 1);
        unstake(&mut app, &alice, &staker_addr, 15000).unwrap();

        let claimable_amount = claimable_amount(&app, &alice, &staker_addr);
        assert!(claimable_amount.u128() == 20000);
    }

    #[test]
    fn test_max_withdraw_after_slashing() {
        let owner = "owner".into_bech32();
        let (mut app, staker_addr, validator_addr) =
            instantiate_staker_with_min_deposit(owner.clone(), "treasury".into_bech32(), 0);

        // mint some INJ tokens to alice
        let alice: Addr = "alice".into_bech32();
        mint_inj(&mut app, &alice, 100_000_000);
        whitelist_user(&mut app, &staker_addr, &owner, &alice);

        // alice stakes 100_000
        stake(&mut app, &alice, &staker_addr, 100_000).unwrap();

        // alice unstakes 50_000
        unstake_when_rewards_accrue(&mut app, &alice, &staker_addr, 50_000, &validator_addr)
            .unwrap();

        // slash the validator by 50%
        let pre_max_withdraw = get_max_withdraw(&app, &staker_addr, &alice);
        app.sudo(cw_multi_test::SudoMsg::Staking(StakingSudo::Slash {
            validator: validator_addr.to_string(),
            percentage: Decimal::percent(50),
        }))
        .unwrap();

        // verify alice max withdraw is now half the pre-slash amount
        assert_eq!(
            get_max_withdraw(&app, &staker_addr, &alice),
            pre_max_withdraw / 2
        );
    }
}
