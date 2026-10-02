# TruStake on Injective

The TruFin INJ staking vault offers users a reliable way of staking INJ on the Injective network.
On staking INJ via the vault, users receive a receipt in the form of the **reward-bearing TruINJ token**.
In addition to the liquid staking functionality, the TruFin staker supports delegating to different validators.

## Whitelist

Users of our vault must be whitelisted to ensure they have completed offline AML/KYC checks and other onboarding requirements.
The contract will verify if the user is included in our whitelist at the time operations such as staking, unstaking, and others are performed.
The use of a whitelist grants TruFin permission to revoke a whitelist status for a malicious user in order to protect the overall integrity of the protocol.

The whitelist is managed by **agents**. Any existing agent may add or remove other agents (the caller must already be an agent). Agent management is a role distinct from the contract `owner`; adding or removing an agent does not change ownership or put any user funds at risk.

## Multi-validator support

The `injective-staker` contract supports the addition of multiple validators.
This allows users to choose which validator they want to delegate to.
By design, users are allowed to deposit into any enabled validator and withdraw from any validator with sufficient funds on it.
The price of TruINJ (aka the share price) is function of the total staked across all validators.

**Notes:**
Validators can be disabled by the admin account but not deleted.

## Known Cosmos unbonding-entry limit (`max_entries = 7`)

Cosmos chains enforce a per-delegator/per-validator cap on concurrent unbonding entries (`max_entries`, currently `7` on Injective). This is a documented chain-level parameter inherited by all pooled delegators and is not a bug in this contract or in Injective. Because the vault delegates as a single account, that cap is shared by all TruINJ holders on a given validator rather than applying per user.

Implications for this vault:
- A validator can temporarily reject new unstakes once all 7 entries are in use, whether they filled through ordinary use or because a whitelisted participant chose to fill them.
- There is no minimum unstake amount, by design, so a user can always exit their full position. It follows that entries can be consumed by arbitrarily small unstakes, and that a participant willing to lock a small amount of INJ can saturate the entries on one validator or on every validator holding stake.
- This does not cause loss of funds: the chain rejects the undelegation before any balance changes, so the unstake reverts with the user's shares and `max_withdraw` accounting preserved.
- Capacity restores automatically as existing unbonding entries mature.

We treat this as a known and accepted operational property of pooled liquid staking. What bounds it is our permissioning rather than its cost: all users are KYC'd institutional participants known to us by identity, and whitelist status can be revoked, which stops an actor from renewing the condition. Should unstake capacity be exhausted, we can also restore it without waiting for entries to mature, by enabling an additional validator and funding it, either through `Redelegate` or from new deposits. Redelegation entries are tracked separately from unbonding entries and redelegated stake stays bonded, so a newly funded validator offers unused unstake capacity immediately.

## Extra security features

### Pausability

The contract is pausable which allows an admin, called the owner, to temporarily prevent anyone from interacting with the contract.
This is useful in case of an emergency where the contract needs to be stopped while a remediation is pending.

### 2-step owner

Replacing the owner is a two-step process, where the new owner account is added as pending and it has to be claimed by the new owner to complete the transfer of ownership.
This prevents adding an invalid owner, which would render the contract without any owner.

## Note on minimum deposits

We require users to stake a minimum of 1 INJ every time.
As we're dealing with institutional clients, we don't expect this to be a problem.
By design, there is no maximum limit to how much can be deposited by a single user.

## Note on rounding errors

There are situations where rounding errors can lead to bad UX.
Consider the example of a user who stakes 100 INJ and then finds out they can only withdraw 99.99999999 INJ. In such cases, we ensure the user can withdraw the full 100 INJ by covering the difference ourselves.
As a general practice, we allow for a few attoINJ to account for rounding errors if it improves the user experience.
To cover costs associated with rounding errors, we ensure the staker account is funded with sufficient INJ.

## Note on fees

The Treasury is an account controlled by TruFin that receives a specified percentage of all rewards. However, instead of sending these rewards to the Treasury, we mint the equivalent amount of TruINJ so that the Treasury can also benefit from staking rewards.
The share price is calculated to already reflect this in order to avoid share price fluctuations when minting TruINJ for the Treasury.

The fee is minted when rewards are realized into the contract, and always on exactly the rewards that operation realizes: `internal_stake` and `internal_unstake` mint on the rewards returned by the validator they act on, while the permissionless `CompoundRewards` mints on every validator's rewards because it withdraws from all of them. Rewards still pending on validators an operation did not touch therefore have their fee reserved rather than collected: the share price already values those rewards net of the fee (`total_rewards * (FEE_PRECISION - fee)`), so the Treasury's share is never distributed to TruINJ holders, and it is minted once a later operation realizes them. Matching the fee basis to the rewards actually realized is what stops the same rewards being charged a fee twice.

## Note on restaking

We run an off-chain process to periodically restake rewards sitting on the validators, and those that were sent to the contract during staking and unstaking operations.

## Note on the `CONTRACT_REWARDS` accounting

`CONTRACT_REWARDS` tracks the liquid INJ that has already been swept into the contract and is awaiting restaking. It is **not** a global tally of all validator rewards: rewards still accruing on the validators are queried live and counted separately when computing the share price, so the two are never double-counted. When a user stakes, `internal_stake` sweeps the current `CONTRACT_REWARDS` into the delegation and then resets `CONTRACT_REWARDS` to the validator's liquid rewards returned during that operation; this reset is intentional, not an overwrite. The on-chain `CompoundRewards` entry point that restakes accrued rewards is **permissionless** by design — anyone may call it — so rewards never sit idle waiting on a privileged caller.

## Note on validator slashing
If one of the configured validators incurs a slashing event, the share price will decrease by an amount proportional to the total stake lost.
Should slashing occur, any stake in the process of unbonding is also subject to the penalty.
In such cases, our Staker might not hold enough assets to fulfill a withdrawal request immediately, which will result in an error reported to the user.
To safeguard against these scenarios, we have agreements in place to top up assets in our Staker, ensuring that these requests can be fulfilled as soon as possible.
That said, we anticipate slashing events to be infrequent and minimally impact our users.

# Developer info

## Prerequisites
Before starting, make sure you have [rustup](https://rustup.rs/) along with a
recent `rustc` and `cargo` version installed. Currently, we are testing on 1.58.1+.

And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Building

To build the staker contract:

`make clean && make build`

## Testing

To run unit and integration tests:

`make test`

## Generating JSON Schema

```sh
# auto-generate the json schema
make schema
```

## Checking the contract is a valid CosmWasm contract

If it's not installed, install `cosmwasm-check` with:
```sh
cargo install cosmwasm-check
```

Then, you can run the following command to check the contract is a valid CosmWasm contract:
```sh
make validate
```

## Preparing the Wasm bytecode for production

Before we upload it to a chain, we need to ensure the smallest output size possible,
as this will be included in the body of a transaction. We also want to have a
reproducible build process, so third parties can verify that the uploaded Wasm
code did indeed come from the claimed rust code.

To solve both these issues, run:

```sh
make build-optimized
```

This produces an `artifacts` directory with a `injective_staker.wasm`, as well as
`checksums.txt`, containing the Sha256 hash of the wasm file.
The wasm file is compiled deterministically (anyone else running the same
docker on the same git commit should get the identical file with the same Sha256 hash).
It is also stripped and minimized for upload to a blockchain (we will also
gzip it in the uploading process to make it even smaller).

## Deploying to testnet

To deploy the Staker contract to testet:

1. Build an optimised wasm binary.
```
make clean && make build-optimized
```

2. Store the contract binary and get the `code_id` for the "store" transaction.
```
injectived tx wasm store ./artifacts/injective_staker.wasm \
   --from=inj1nnqkmxk0ujmcvlcegsauutfuf7y4vf9azzecn9 \
   --chain-id=injective-888 \
   --fees=1000000000000000inj \
   --gas=5000000 \
   --node="https://testnet.sentry.tm.injective.network"
```

3. Instantiate the contract.
Make sure to set an admin using the --admin flag. This user will have the ability to migrate the contract if updates need to be made.
```
injectived tx wasm instantiate <code_id> '{"owner": "<owner_addr>", "default_validator": "<validator_addr>", "treasury": "<treasury_addr>"}' \
  --from <signer_wallet_name> \
  --label "TruFin staker" \
  --admin <owner_addr> \
  --gas-prices 4800000000000inj \
  --gas=3000000 \
  --chain-id injective-888 \
  --node "https://testnet.sentry.tm.injective.network"

```

# Upgrading the testnet contract

To upgrade the Staker contract in testnet:

1. Build an optimised wasm binary.
```
make clean && make build-optimized
```

2. Store the new binary and get the new `code_id`.

```
injectived tx wasm store ./artifacts/injective_staker.wasm \
   --from=inj1nnqkmxk0ujmcvlcegsauutfuf7y4vf9azzecn9 \
   --chain-id=injective-888 \
   --fees=1000000000000000inj \
   --gas=5000000 \
   --node="https://testnet.sentry.tm.injective.network"
```
3. Call the `migrate` function of the new binary.
```
injectived tx wasm migrate <contract_addr> <code_id> '{}' \
  --from <owner_addr> \
  --chain-id="injective-888" \
  --fees=8000000000000000inj \
  --gas=50000000 \
  --node="https://testnet.sentry.tm.injective.network:443"
```
