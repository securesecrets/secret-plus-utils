use cosmwasm_std::{
    Decimal, DistributionMsg, Empty, StakingMsg, StakingQuery,
    Validator, FullDelegation,
    Querier, Storage, Binary, BlockInfo, Api, Addr,
    to_binary, AllValidatorsResponse, ValidatorResponse,
    BankMsg, BankQuery, BondedDenomResponse, CustomQuery,
    Coin,
};
use anyhow::{bail, anyhow, Result as AnyResult, Error};
use schemars::JsonSchema;
use secret_storage_plus::{Item};

use crate::{
    app::CosmosRouter,
    executor::AppResponse,
    module::FailingModule,
    Module,
    bank::BankSudo,
};

const VALIDATORS: Item<Vec<String>> = Item::new("validators");
const DELEGATIONS: Item<Vec<FullDelegation>> = Item::new("delegations");

// We need to expand on this, but we will need this to properly test out staking
#[derive(Clone, std::fmt::Debug, PartialEq, JsonSchema)]
pub enum StakingSudo {
    Slash {
        validator: String,
        percentage: Decimal,
    },
}

pub trait Staking: Module<ExecT = StakingMsg, QueryT = StakingQuery, SudoT = StakingSudo> {}

#[derive(Default)]
pub struct StakingKeeper {}

impl StakingKeeper {
    pub fn new() -> Self {
        StakingKeeper {}
    }

    pub fn add_validator(validator: String, storage: &mut dyn Storage) -> Vec<String> {
        let mut validators = VALIDATORS.load(storage).unwrap();
        validators.push(validator);
        VALIDATORS.save(storage, &validators).unwrap();
        validators
    }

    // adds 'amount' to 'accumulated_rewards' for every delegator/validator pair
    pub fn add_rewards<ExecC, QueryC: CustomQuery>(
        &self,
        amount: Coin, 
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        api: &dyn Api,
        storage: &mut dyn Storage,
        block: &BlockInfo,
    ) -> Vec<FullDelegation> {

        let mut delegations = DELEGATIONS.load(storage).unwrap();

        for i in 0..delegations.len() {
            if let Some(mut coin) = delegations[i]
                .accumulated_rewards
                .clone()
                .into_iter()
                .find(|ar| ar.denom == amount.denom.clone()) {
                    coin.amount += amount.amount.clone();
                    break;
            }
            else {
                delegations[i].accumulated_rewards.push(amount.clone());
            }

            let mint = BankSudo::Mint {
                to_address: delegations[i].validator.clone(),
                amount: vec![amount.clone()],
            };
            router.sudo(api, storage, block, mint.into()).unwrap();
        }
        DELEGATIONS.save(storage, &delegations).unwrap();
        delegations
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

                router.execute(
                    api, storage, block, 
                    sender.clone(), 
                    BankMsg::Send {
                        to_address: validator.clone(),
                        amount: vec![amount.clone()],
                    }.into(),
                )?;

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);

                if let Some(i) = delegations
                    .iter()
                    .position(|d| d.delegator == sender &&
                              d.validator.clone() == validator &&
                              d.amount.denom == amount.clone().denom) {
                        delegations[i].amount.amount += amount.clone().amount;
                    }
                else {
                    delegations.push(FullDelegation {
                        delegator: sender,
                        validator: validator.clone(),
                        amount: amount.clone(),
                        can_redelegate: amount.clone(),
                        accumulated_rewards: vec![],
                    });
                }
                DELEGATIONS.save(storage, &delegations)?;

                Ok(AppResponse { events: vec![], data: None })
            }
            StakingMsg::Undelegate { validator, amount } => {

                let mut send_coins = vec![];

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);
                if let Some(i) = delegations
                    .iter()
                    .position(|d| d.delegator == sender && 
                              d.validator.clone() == validator && 
                              d.amount.denom == amount.clone().denom) {
                        delegations[i].amount.amount -= amount.amount.clone();
                        send_coins.push(amount.clone());
                        send_coins.append(&mut delegations[i].accumulated_rewards);
                        delegations[i].accumulated_rewards = vec![];

                        if delegations[i].amount.amount.is_zero() {
                            delegations.remove(i);
                        }
                    }
                else {
                    bail!("Insufficient delegation to undelegate");
                }

                let send = BankMsg::Send {
                    to_address: sender.to_string().clone(),
                    amount: send_coins,
                };
                
                router.execute(api, storage, block, api.addr_validate(&validator.clone())?, send.into())?;
                DELEGATIONS.save(storage, &delegations)?;
                Ok(AppResponse { events: vec![], data: None })
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
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        msg: StakingSudo,
    ) -> AnyResult<AppResponse> {
        match msg {
            StakingSudo::Slash { validator, percentage } => {
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
            StakingQuery::BondedDenom { } => {
                Ok(to_binary(&BondedDenomResponse {
                    denom: "scrt".into(),
                })?)
            }
            StakingQuery::AllDelegations { delegator } => {
                let delegations: Vec<FullDelegation> = DELEGATIONS.load(storage)
                    .unwrap_or(vec![])
                    .into_iter()
                    .filter(|d| d.delegator.to_string() == delegator)
                    .collect();
                Ok(to_binary(&delegations)?)
            }
            StakingQuery::Delegation { delegator, validator } => {
                match DELEGATIONS.load(storage)?
                    .into_iter()
                    .find(|d| d.delegator == delegator && d.validator == validator) {
                        Some(d) => Ok(to_binary(&d)?),
                        None => Err(anyhow!("missing delegator")),
                }
            }
            StakingQuery::AllValidators {} => {
                let validators = VALIDATORS.load(storage)?
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
            StakingQuery::Validator { address } => {
                Ok(to_binary(&ValidatorResponse {
                    validator: match VALIDATORS.load(storage)?
                            .into_iter()
                            .find(|v| *v == address) {
                                Some(v) => Some(Validator {
                                            address: v,
                                            commission: Decimal::zero(),
                                            max_commission: Decimal::one(),
                                            max_change_rate: Decimal::one(),
                                        }),
                                None => None,
                            }

                })?)
            }
            q => bail!("Unsupported bank query: {:?}", q),
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

    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: DistributionMsg,
    ) -> AnyResult<AppResponse> {
        match msg {
            DistributionMsg::WithdrawDelegatorReward { validator } => {
                /*
                let send = BankMsg::Send {
                    to_address: validator.clone(),
                    amount: vec![amount.clone()],
                };
                router.execute(api, storage, block, sender.clone(), send.into())?;

                let mut delegations = DELEGATIONS.load(storage)?;
                if let Some(i) = delegations
                    .iter()
                    .position(|d| d.delegator == sender && 
                              d.validator.clone() == validator && 
                              d.amount.denom == amount.clone().denom) {
                        delegations[i].amount.amount += amount.clone().amount;
                    }
                else {
                    delegations.push(FullDelegation {
                        delegator: sender,
                        validator: validator.clone(),
                        amount: amount.clone(),
                        can_redelegate: amount.clone(),
                        accumulated_rewards: vec![],
                    });
                }
                DELEGATIONS.save(storage, &delegations)?;
                */
                bail!("WithdrawDelegatorReward Not Implemented");

                Ok(AppResponse { events: vec![], data: None })
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
        app::{MockRouter, Router},
        wasm::WasmKeeper,
        bank::{Bank, BankKeeper, BankSudo},
    };
    use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{Coin, coins, from_slice, Empty, StdError, Uint128, from_binary};

    type BasicRouter<ExecC = Empty, QueryC = Empty> = Router<
        BankKeeper,
        FailingModule<ExecC, QueryC, Empty>,
        WasmKeeper<ExecC, QueryC>,
        StakingKeeper,
        DistributionKeeper,
    >;

    fn mock_router() -> BasicRouter {
        Router {
            wasm: WasmKeeper::new(),
            bank: BankKeeper::new(),
            custom: FailingModule::new(),
            staking: StakingKeeper::new(),
            distribution: DistributionKeeper::new(),
        }
    }

    #[test]
    fn staking() {
        let api = MockApi::default();
        let mut storage = MockStorage::new();
        let block = mock_env().block;
        let querier: MockQuerier<Empty> = MockQuerier::new(&[]);

        let owner = Addr::unchecked("owner");
        let validator = Addr::unchecked("validator");
        //let rcpt = Addr::unchecked("receiver");
        let funds = Coin { 
            amount: Uint128::new(100),
            denom: "eth".into()
        };
        let rewards = Coin { 
            amount: Uint128::new(10),
            denom: "eth".into()
        };
        //let norm = vec![coin(20, "btc"), coin(100, "eth")];
        let bank = BankKeeper::new();
        let staking = StakingKeeper::new();
        let router = mock_router();

        let mut expected_delegation = FullDelegation {
            delegator: owner.clone(),
            validator: validator.to_string(),
            amount: funds.clone(),
            can_redelegate: funds.clone(),
            accumulated_rewards: vec![],
        };

        bank.init_balance(&mut storage, &owner, vec![funds.clone()]).unwrap();

        staking.execute(
            &api,
            &mut storage,
            &router,
            &block,
            owner.clone(),
            StakingMsg::Delegate {
                validator: validator.clone().into(), 
                amount: funds.clone(),
            },
        ).unwrap();

        let delegation: FullDelegation = from_binary(&staking.query(
            &api,
            &storage,
            &querier,
            &block,
            StakingQuery::Delegation {
                delegator: owner.to_string(),
                validator: validator.to_string(),
            },
        ).unwrap()).unwrap();

        assert_eq!(delegation, expected_delegation);

        staking.add_rewards(
            rewards.clone(),
            &router,
            &api,
            &mut storage,
            &block
        );
        let delegation: FullDelegation = from_binary(&staking.query(
            &api,
            &storage,
            &querier,
            &block,
            StakingQuery::Delegation {
                delegator: owner.to_string(),
                validator: validator.to_string(),
            },
        ).unwrap()).unwrap();
        expected_delegation.accumulated_rewards.push(rewards.clone());
        assert_eq!(delegation, expected_delegation);

        staking.execute(
            &api,
            &mut storage,
            &router,
            &block,
            owner.clone(),
            StakingMsg::Undelegate {
                validator: validator.clone().into(), 
                amount: funds.clone(),
            },
        ).unwrap();

        let delegation: FullDelegation = from_binary(&staking.query(
            &api,
            &storage,
            &querier,
            &block,
            StakingQuery::Delegation {
                delegator: owner.to_string(),
                validator: validator.to_string(),
            },
        ).unwrap()).unwrap();
        //expected_delegation.amount.push(rewards.clone());
        assert_eq!(delegation, expected_delegation);
    }
}
