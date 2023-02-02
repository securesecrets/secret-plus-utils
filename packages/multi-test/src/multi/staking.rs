use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{
    to_binary, Addr, AllDelegationsResponse, AllValidatorsResponse, Api, BankMsg, Binary,
    BlockInfo, BondedDenomResponse, Coin, CustomQuery, Decimal, Delegation, DelegationResponse,
    DistributionMsg, Empty, FullDelegation, Querier, StakingMsg, StakingQuery, Storage, Validator,
    ValidatorResponse,
};
use schemars::JsonSchema;
use secret_storage_plus::Item;

use crate::{
    app::{CosmosRouter, Router},
    bank::{BankKeeper, BankSudo},
    executor::AppResponse,
    wasm::WasmKeeper,
    Module,
};

const VALIDATORS: Item<Vec<String>> = Item::new("validators");
const DELEGATIONS: Item<Vec<FullDelegation>> = Item::new("delegations");
// validator, user, coins
const UNDELEGATIONS: Item<Vec<(Addr, Addr, Vec<Coin>)>> = Item::new("undelegations");

const BONDED_DENOM: &str = "uscrt";

// We need to expand on this, but we will need this to properly test out staking
#[derive(Clone, std::fmt::Debug, PartialEq, JsonSchema)]
pub enum StakingSudo {
    Slash {
        validator: String,
        percentage: Decimal,
    },
    AddValidator {
        validator: String,
    },
    AddRewards {
        amount: Coin,
    },
    FastForwardUndelegate {},
}

pub trait Staking: Module<ExecT = StakingMsg, QueryT = StakingQuery, SudoT = StakingSudo> {}

#[derive(Default)]
pub struct StakingKeeper {}

impl StakingKeeper {
    pub fn new() -> Self {
        StakingKeeper {}
    }
}

impl Staking for StakingKeeper {}

impl Module for StakingKeeper {
    type ExecT = StakingMsg;
    type QueryT = StakingQuery;
    type SudoT = StakingSudo;

    fn execute<ExecC, QueryC: CustomQuery>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: StakingMsg,
    ) -> AnyResult<AppResponse> {
        match msg {
            StakingMsg::Delegate { validator, amount } => {
                println!("Delegating {}", amount);

                router.execute(
                    api,
                    storage,
                    block,
                    sender.clone(),
                    BankMsg::Send {
                        to_address: validator.clone(),
                        amount: vec![amount.clone()],
                    }
                    .into(),
                )?;

                if VALIDATORS
                    .load(storage)
                    .unwrap_or(vec![])
                    .into_iter()
                    .find(|v| *v == validator)
                    .is_none()
                {
                    bail!("Validator {} not found", validator);
                }

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);

                if let Some(i) = delegations.iter().position(|d| {
                    d.delegator == sender
                        && d.validator.clone() == validator
                        && d.amount.denom == amount.clone().denom
                }) {
                    delegations[i].amount.amount += amount.amount.clone();
                } else {
                    delegations.push(FullDelegation {
                        delegator: sender,
                        validator: validator.clone(),
                        amount: amount.clone(),
                        can_redelegate: amount.clone(),
                        accumulated_rewards: vec![],
                    });
                }
                DELEGATIONS.save(storage, &delegations)?;
                println!(
                    "Post Save Delegations {}",
                    DELEGATIONS.load(storage).unwrap().len()
                );

                Ok(AppResponse {
                    events: vec![],
                    data: None,
                })
            }
            StakingMsg::Undelegate { validator, amount } => {
                println!("Undelegating {}", amount);

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);
                if let Some(i) = delegations.iter().position(|d| {
                    d.delegator == sender
                        && d.validator.clone() == validator
                        && d.amount.denom == amount.denom.clone()
                }) {
                    if !delegations[i].accumulated_rewards.is_empty() {
                        router
                            .sudo(
                                api,
                                storage,
                                block,
                                BankSudo::Mint {
                                    to_address: sender.to_string(),
                                    amount: delegations[i].accumulated_rewards.clone(),
                                }
                                .into(),
                            )
                            .unwrap();
                    }

                    let mut undelegations = UNDELEGATIONS.load(storage).unwrap_or(vec![]);
                    undelegations.push((
                        Addr::unchecked(validator),
                        sender,
                        vec![delegations[i].amount.clone()],
                    ));
                    UNDELEGATIONS.save(storage, &undelegations)?;

                    delegations[i].amount.amount -= amount.amount.clone();
                    delegations[i].accumulated_rewards = vec![];

                    if delegations[i].amount.amount.is_zero() {
                        delegations.remove(i);
                    }

                    DELEGATIONS.save(storage, &delegations)?;
                } else {
                    bail!("Insufficient delegation to undelegate");
                }

                Ok(AppResponse {
                    events: vec![],
                    data: None,
                })
            }
            /*
            StakingMsg::Redelegate { src_validator, dst_validator, amount } => {
                Ok(AppResponse { events: vec![], data: None })
            }
            */
            m => bail!("Unsupported staking message: {:?}", m),
        }
    }

    fn sudo<ExecC, QueryC: CustomQuery>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        msg: StakingSudo,
    ) -> AnyResult<AppResponse> {
        match msg {
            StakingSudo::Slash {
                validator,
                percentage,
            } => {
                bail!("slashing not implemented");
                Ok(AppResponse::default())
            }
            StakingSudo::AddValidator { validator } => {
                let mut validators = VALIDATORS.load(storage).unwrap_or(vec![]);
                validators.push(validator);
                VALIDATORS.save(storage, &validators)?;
                Ok(AppResponse::default())
            }
            StakingSudo::AddRewards { amount } => {
                if amount.amount.is_zero() {
                    return Ok(AppResponse::default());
                }

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);

                for i in 0..delegations.len() {
                    router
                        .sudo(
                            api,
                            storage,
                            block,
                            BankSudo::Mint {
                                to_address: delegations[i].validator.to_string(),
                                amount: vec![amount.clone()],
                            }
                            .into(),
                        )
                        .unwrap();

                    if let Some(mut coin) = delegations[i]
                        .accumulated_rewards
                        .clone()
                        .into_iter()
                        .find(|ar| ar.denom == amount.denom.clone())
                    {
                        coin.amount += amount.amount.clone();
                        break;
                    } else {
                        delegations[i].accumulated_rewards.push(amount.clone());
                    }
                }
                DELEGATIONS.save(storage, &delegations)?;
                Ok(AppResponse::default())
            }
            StakingSudo::FastForwardUndelegate {} => {
                for (validator, user, coins) in UNDELEGATIONS.load(storage).unwrap_or(vec![]) {
                    //router.bank.send(storage, validator, user, coins)?;
                    router.execute(
                        api,
                        storage,
                        block,
                        validator,
                        BankMsg::Send {
                            to_address: user.to_string().clone(),
                            amount: coins,
                        }
                        .into(),
                    )?;
                }
                UNDELEGATIONS.save(storage, &vec![])?;
                Ok(AppResponse::default())
            }
        }
    }

    fn query(
        &self,
        api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: StakingQuery,
    ) -> AnyResult<Binary> {
        //let bank_storage = prefixed_read(storage, NAMESPACE_BANK);
        match request {
            StakingQuery::BondedDenom {} => Ok(to_binary(&BondedDenomResponse {
                denom: BONDED_DENOM.into(),
            })?),
            StakingQuery::AllDelegations { delegator } => {
                let delegations: Vec<Delegation> = DELEGATIONS
                    .load(storage)
                    .unwrap_or(vec![])
                    .into_iter()
                    .filter(|d| d.delegator.to_string() == delegator)
                    .map(|d| Delegation {
                        delegator: d.delegator,
                        validator: d.validator,
                        amount: d.amount,
                    })
                    .collect();
                //println!("Querying delegations {}", delegations.len());
                //assert!(delegations.len() > 0);
                Ok(to_binary(&AllDelegationsResponse { delegations })?)
            }
            StakingQuery::Delegation {
                delegator,
                validator,
            } => {
                let delegations = DELEGATIONS.load(storage)?;
                //let d = delegations.into_iter().find(|d| d.delegator == delegator && d.validator == validator).unwrap();
                /*
                if let Some(d) = DELEGATIONS.load(storage)
                        .unwrap_or(vec![])
                        .into_iter()
                        .find(|d| d.delegator == delegator && d.validator == validator) {

                    println!("Found delegation validator: {} delegator: {} amount: {}", d.validator, d.delegator, d.amount);
                }
                else {
                    println!("No Match! {} {}", validator, delegator);
                }
                */

                Ok(to_binary(&DelegationResponse {
                    delegation: DELEGATIONS
                        .load(storage)
                        .unwrap_or(vec![])
                        .into_iter()
                        .find(|d| d.delegator == delegator && d.validator == validator),
                })?)
            }
            StakingQuery::AllValidators {} => {
                let validators = VALIDATORS
                    .load(storage)
                    .unwrap_or(vec![])
                    .into_iter()
                    .map(|v| Validator {
                        address: v,
                        commission: Decimal::zero(),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
                    })
                    .collect();
                Ok(to_binary(&AllValidatorsResponse { validators })?)
            }
            StakingQuery::Validator { address } => Ok(to_binary(&ValidatorResponse {
                validator: match VALIDATORS
                    .load(storage)
                    .unwrap_or(vec![])
                    .into_iter()
                    .find(|v| *v == address)
                {
                    Some(v) => Some(Validator {
                        address: v,
                        commission: Decimal::zero(),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
                    }),
                    None => None,
                },
            })?),
            q => bail!("Unsupported staking query: {:?}", q),
        }
    }
}

/*
#[derive(Clone, std::fmt::Debug, PartialEq, JsonSchema)]
pub enum DistributionSudo { }
*/

pub trait Distribution: Module<ExecT = DistributionMsg, QueryT = Empty, SudoT = Empty> {}

#[derive(Default)]
pub struct DistributionKeeper {}

impl DistributionKeeper {
    pub fn new() -> Self {
        DistributionKeeper {}
    }
}

impl Distribution for DistributionKeeper {}

impl Module for DistributionKeeper {
    type ExecT = DistributionMsg;
    type QueryT = Empty;
    type SudoT = Empty;

    fn execute<ExecC, QueryC: CustomQuery>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: DistributionMsg,
    ) -> AnyResult<AppResponse> {
        println!("dist msg");

        match msg {
            DistributionMsg::WithdrawDelegatorReward { validator } => {
                println!(
                    "trying to withdraw {} {}",
                    validator.clone(),
                    sender.clone()
                );

                let mut delegations = DELEGATIONS.load(storage)?;

                if let Some(i) = delegations
                    .iter()
                    .position(|d| d.delegator == sender && d.validator.clone() == validator)
                {
                    println!(
                        "withdraw delegation {} {}",
                        delegations[i].validator, sender
                    );
                    if !delegations[i].accumulated_rewards.is_empty() {
                        println!(
                            "Withdraw Rewards {} {}",
                            delegations[i].accumulated_rewards[0].amount,
                            delegations[i].accumulated_rewards[0].denom
                        );
                        router
                            .sudo(
                                api,
                                storage,
                                block,
                                BankSudo::Mint {
                                    to_address: sender.to_string(),
                                    amount: delegations[i].accumulated_rewards.clone(),
                                }
                                .into(),
                            )
                            .unwrap();
                    }
                    delegations[i].accumulated_rewards = vec![];
                    DELEGATIONS.save(storage, &delegations)?;
                } else {
                    bail!(
                        "{} has no rewards with {}",
                        sender.clone(),
                        validator.clone()
                    );
                }

                Ok(AppResponse {
                    events: vec![],
                    data: None,
                })
            }
            m => bail!("Unsupported distribution message: {:?}", m),
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        msg: Empty,
    ) -> AnyResult<AppResponse> {
        bail!("Unsupported distribution sudo: {:?}", msg);
    }

    fn query(
        &self,
        api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Empty,
    ) -> AnyResult<Binary> {
        bail!("Unsupported distribution query: {:?}", request);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        bank::{Bank, BankKeeper, BankSudo},
        test_helpers::mocks::{mock_router, BasicRouter},
        wasm::WasmKeeper,
    };
    use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, from_binary, from_slice, Coin, Empty, StdError, Uint128};

    #[test]
    fn staking() {
        let api = MockApi::default();
        let mut storage = MockStorage::new();
        let block = mock_env().block;
        let querier: MockQuerier<Empty> = MockQuerier::new(&[]);

        let owner = Addr::unchecked("owner");
        let validator = Addr::unchecked("validator");

        let funds = Coin {
            amount: Uint128::new(100),
            denom: "eth".into(),
        };
        let rewards = Coin {
            amount: Uint128::new(10),
            denom: "eth".into(),
        };
        //let norm = vec![coin(20, "btc"), coin(100, "eth")];
        let bank = BankKeeper::new();
        let staking = StakingKeeper::new();
        let router = mock_router();

        staking
            .sudo(
                &api,
                &mut storage,
                &router,
                &block,
                StakingSudo::AddValidator {
                    validator: validator.to_string(),
                },
            )
            .unwrap();

        let mut expected_delegation = FullDelegation {
            delegator: owner.clone(),
            validator: validator.to_string(),
            amount: funds.clone(),
            can_redelegate: funds.clone(),
            accumulated_rewards: vec![],
        };

        bank.init_balance(&mut storage, &owner, vec![funds.clone()])
            .unwrap();

        staking
            .execute(
                &api,
                &mut storage,
                &router,
                &block,
                owner.clone(),
                StakingMsg::Delegate {
                    validator: validator.clone().into(),
                    amount: funds.clone(),
                },
            )
            .unwrap();

        let delegation: DelegationResponse = from_binary(
            &staking
                .query(
                    &api,
                    &storage,
                    &querier,
                    &block,
                    StakingQuery::Delegation {
                        delegator: owner.to_string(),
                        validator: validator.to_string(),
                    },
                )
                .unwrap(),
        )
        .unwrap();

        assert_eq!(delegation.delegation.unwrap(), expected_delegation);

        let delegations: AllDelegationsResponse = from_binary(
            &staking
                .query(
                    &api,
                    &storage,
                    &querier,
                    &block,
                    StakingQuery::AllDelegations {
                        delegator: owner.to_string(),
                        //validator: validator.to_string(),
                    },
                )
                .unwrap(),
        )
        .unwrap();

        assert_eq!(delegations.delegations.len(), 1);

        staking
            .sudo(
                &api,
                &mut storage,
                &router,
                &block,
                StakingSudo::AddRewards {
                    amount: rewards.clone(),
                },
            )
            .unwrap();

        let delegation: DelegationResponse = from_binary(
            &staking
                .query(
                    &api,
                    &storage,
                    &querier,
                    &block,
                    StakingQuery::Delegation {
                        delegator: owner.to_string(),
                        validator: validator.to_string(),
                    },
                )
                .unwrap(),
        )
        .unwrap();

        //expected_delegation.accumulated_rewards.push(rewards.clone());
        assert_eq!(
            delegation.delegation.unwrap().accumulated_rewards,
            vec![rewards.clone()]
        );

        staking
            .execute(
                &api,
                &mut storage,
                &router,
                &block,
                owner.clone(),
                StakingMsg::Undelegate {
                    validator: validator.clone().into(),
                    amount: funds.clone(),
                },
            )
            .unwrap();

        let delegation: DelegationResponse = from_binary(
            &staking
                .query(
                    &api,
                    &storage,
                    &querier,
                    &block,
                    StakingQuery::Delegation {
                        delegator: owner.to_string(),
                        validator: validator.to_string(),
                    },
                )
                .unwrap(),
        )
        .unwrap();
        //expected_delegation.amount.push(rewards.clone());
        assert!(delegation.delegation.is_none());
    }
}
