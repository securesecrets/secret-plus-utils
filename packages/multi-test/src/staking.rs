use cosmwasm_std::{
    Decimal, DistributionMsg, Empty, StakingMsg, StakingQuery,
    Validator, FullDelegation, 
    Querier, Storage, Binary, BlockInfo, Api, Addr,
    to_binary, AllValidatorsResponse, ValidatorResponse,
    BankMsg, BankQuery, BondedDenomResponse,
    //Delegation, Coin,
};
use anyhow::{bail, Result as AnyResult};
use schemars::JsonSchema;
use secret_storage_plus::{Item};

use crate::{
    app::CosmosRouter,
    executor::AppResponse,
    module::FailingModule,
    Module,
    bank::{Bank, BankKeeper, BankSudo},
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

    /*
    pub fn add_validator(validator: String, storage: &mut dyn Storage) -> Self {
        VALIDATORS.update(storage, |validators| {
            validators.push(validator);
            Ok(validators)
        })?;
    }

    pub fn add_rewards(amount: Coin, storage: &mut dyn Storage) -> Self {
        let mut delegations = DELEGATIONS.load(storage);

        for i in 0..delegations.len() {
            if let Some(coin) = delegations[i].accumulated_rewards
                .iter()
                .find(|ar| ar.denom == amount.denom) {
                    coin.amount += amount.amount;
            }
            else {
                delegations[i].accumulated_rewards.push(amount);
            }
        }
        DELEGATIONS.save(storage, &delegations);
    }
    */
}

impl Staking for StakingKeeper {}

impl Module for StakingKeeper {
    type ExecT = StakingMsg;
    type QueryT = StakingQuery;
    type SudoT = StakingSudo;

    fn execute<ExecC, QueryC>(
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

                Ok(AppResponse { events: vec![], data: None })
            }
            StakingMsg::Undelegate { validator, amount } => {
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

    fn sudo<ExecC, QueryC>(
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
                let delegations: Vec<FullDelegation> = DELEGATIONS.load(storage)?
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
                        None => bail!("failed to find delegation")
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

pub type FailingStaking = FailingModule<StakingMsg, StakingQuery, StakingSudo>;

impl Staking for FailingStaking {}

pub trait Distribution: Module<ExecT = DistributionMsg, QueryT = Empty, SudoT = Empty> {}

pub type FailingDistribution = FailingModule<DistributionMsg, Empty, Empty>;

impl Distribution for FailingDistribution {}
