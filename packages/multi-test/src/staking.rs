use cosmwasm_std::{
    Decimal, DistributionMsg, Empty, StakingMsg, StakingQuery,
    Validator, FullDelegation,
    Querier, Storage, Binary, BlockInfo, Api, Addr,
    to_binary, AllValidatorsResponse, ValidatorResponse,
    BankMsg, BankQuery, BondedDenomResponse, CustomQuery,
    Coin,
};
use anyhow::{bail, Result as AnyResult};
use schemars::JsonSchema;
use secret_storage_plus::{Item};

use crate::{
    app::CosmosRouter,
    executor::AppResponse,
    module::FailingModule,
    Module,
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

    pub fn add_rewards(amount: Coin, storage: &mut dyn Storage) -> Vec<FullDelegation> {
        let mut delegations = DELEGATIONS.load(storage).unwrap();

        for i in 0..delegations.len() {
            if let Some(mut coin) = delegations[i].accumulated_rewards.clone()
                .into_iter()
                .find(|ar| ar.denom == amount.denom.clone()) {
                    coin.amount += amount.amount.clone();
                    break;
            }
            else {
                delegations[i].accumulated_rewards.push(amount.clone());
            }
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
                let send = BankMsg::Send {
                    to_address: validator.clone(),
                    amount: vec![amount.clone()],
                };
                router.execute(api, storage, block, sender.clone(), send.into())?;

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
                let send = BankMsg::Send {
                    to_address: sender.to_string().clone(),
                    amount: vec![amount.clone()],
                };
                
                router.execute(api, storage, block, api.addr_validate(&validator.clone())?, send.into())?;

                let mut delegations = DELEGATIONS.load(storage).unwrap_or(vec![]);
                if let Some(i) = delegations
                    .iter()
                    .position(|d| d.delegator == sender && 
                              d.validator.clone() == validator && 
                              d.amount.denom == amount.clone().denom) {
                        delegations[i].amount.amount -= amount.clone().amount;
                    }
                else {
                    bail!("Insufficient delegation to undelegate");
                }
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
    fn delegate() {
        let api = MockApi::default();
        let mut store = MockStorage::new();
        let block = mock_env().block;
        let querier: MockQuerier<Empty> = MockQuerier::new(&[]);

        let owner = Addr::unchecked("owner");
        let validator = Addr::unchecked("validator");
        //let rcpt = Addr::unchecked("receiver");
        let funds = Coin { 
            amount: Uint128::new(100),
            denom: "eth".into()
        };
        //let norm = vec![coin(20, "btc"), coin(100, "eth")];
        let bank = BankKeeper::new();
        let staking = StakingKeeper::new();
        let router = mock_router();

        bank.init_balance(&mut store, &owner, vec![funds.clone()]).unwrap();

        staking.execute(
            &api,
            &mut store,
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
            &store,
            &querier,
            &block,
            StakingQuery::Delegation {
                delegator: owner.to_string(),
                validator: validator.to_string(),
            },
        ).unwrap()).unwrap();

        assert_eq!(delegation.delegator, owner);
        assert_eq!(delegation.validator, validator);
        assert_eq!(delegation.amount, funds);
        /*
        let res: AllBalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(res.amount, norm);

        let req = BankQuery::AllBalances {
            address: rcpt.clone().into(),
        };
        let raw = bank.query(&api, &store, &querier, &block, req).unwrap();
        let res: AllBalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(res.amount, vec![]);

        let req = BankQuery::Balance {
            address: owner.clone().into(),
            denom: "eth".into(),
        };
        let raw = bank.query(&api, &store, &querier, &block, req).unwrap();
        let res: BalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(res.amount, coin(100, "eth"));

        let req = BankQuery::Balance {
            address: owner.into(),
            denom: "foobar".into(),
        };
        let raw = bank.query(&api, &store, &querier, &block, req).unwrap();
        let res: BalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(res.amount, coin(0, "foobar"));

        let req = BankQuery::Balance {
            address: rcpt.into(),
            denom: "eth".into(),
        };
        let raw = bank.query(&api, &store, &querier, &block, req).unwrap();
        let res: BalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(res.amount, coin(0, "eth"));
        */
    }

    /*
    #[test]
    fn send_coins() {
        let api = MockApi::default();
        let mut store = MockStorage::new();
        let block = mock_env().block;
        let router = MockRouter::default();

        let owner = Addr::unchecked("owner");
        let rcpt = Addr::unchecked("receiver");
        let init_funds = vec![coin(20, "btc"), coin(100, "eth")];
        let rcpt_funds = vec![coin(5, "btc")];

        // set money
        let bank = BankKeeper::new();
        bank.init_balance(&mut store, &owner, init_funds).unwrap();
        bank.init_balance(&mut store, &rcpt, rcpt_funds).unwrap();

        // send both tokens
        let to_send = vec![coin(30, "eth"), coin(5, "btc")];
        let msg = BankMsg::Send {
            to_address: rcpt.clone().into(),
            amount: to_send,
        };
        bank.execute(
            &api,
            &mut store,
            &router,
            &block,
            owner.clone(),
            msg.clone(),
        )
        .unwrap();
        let rich = query_balance(&bank, &api, &store, &owner);
        assert_eq!(vec![coin(15, "btc"), coin(70, "eth")], rich);
        let poor = query_balance(&bank, &api, &store, &rcpt);
        assert_eq!(vec![coin(10, "btc"), coin(30, "eth")], poor);

        // can send from any account with funds
        bank.execute(&api, &mut store, &router, &block, rcpt.clone(), msg)
            .unwrap();

        // cannot send too much
        let msg = BankMsg::Send {
            to_address: rcpt.into(),
            amount: coins(20, "btc"),
        };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap_err();

        let rich = query_balance(&bank, &api, &store, &owner);
        assert_eq!(vec![coin(15, "btc"), coin(70, "eth")], rich);
    }

    #[test]
    fn burn_coins() {
        let api = MockApi::default();
        let mut store = MockStorage::new();
        let block = mock_env().block;
        let router = MockRouter::default();

        let owner = Addr::unchecked("owner");
        let rcpt = Addr::unchecked("recipient");
        let init_funds = vec![coin(20, "btc"), coin(100, "eth")];

        // set money
        let bank = BankKeeper::new();
        bank.init_balance(&mut store, &owner, init_funds).unwrap();

        // burn both tokens
        let to_burn = vec![coin(30, "eth"), coin(5, "btc")];
        let msg = BankMsg::Burn { amount: to_burn };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap();
        let rich = query_balance(&bank, &api, &store, &owner);
        assert_eq!(vec![coin(15, "btc"), coin(70, "eth")], rich);

        // cannot burn too much
        let msg = BankMsg::Burn {
            amount: coins(20, "btc"),
        };
        let err = bank
            .execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap_err();
        assert!(matches!(err.downcast().unwrap(), StdError::Overflow { .. }));

        let rich = query_balance(&bank, &api, &store, &owner);
        assert_eq!(vec![coin(15, "btc"), coin(70, "eth")], rich);

        // cannot burn from empty account
        let msg = BankMsg::Burn {
            amount: coins(1, "btc"),
        };
        let err = bank
            .execute(&api, &mut store, &router, &block, rcpt, msg)
            .unwrap_err();
        assert!(matches!(err.downcast().unwrap(), StdError::Overflow { .. }));
    }

    #[test]
    fn fail_on_zero_values() {
        let api = MockApi::default();
        let mut store = MockStorage::new();
        let block = mock_env().block;
        let router = MockRouter::default();

        let owner = Addr::unchecked("owner");
        let rcpt = Addr::unchecked("recipient");
        let init_funds = vec![coin(5000, "atom"), coin(100, "eth")];

        // set money
        let bank = BankKeeper::new();
        bank.init_balance(&mut store, &owner, init_funds).unwrap();

        // can send normal amounts
        let msg = BankMsg::Send {
            to_address: rcpt.to_string(),
            amount: coins(100, "atom"),
        };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap();

        // fails send on no coins
        let msg = BankMsg::Send {
            to_address: rcpt.to_string(),
            amount: vec![],
        };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap_err();

        // fails send on 0 coins
        let msg = BankMsg::Send {
            to_address: rcpt.to_string(),
            amount: coins(0, "atom"),
        };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap_err();

        // fails burn on no coins
        let msg = BankMsg::Burn { amount: vec![] };
        bank.execute(&api, &mut store, &router, &block, owner.clone(), msg)
            .unwrap_err();

        // fails burn on 0 coins
        let msg = BankMsg::Burn {
            amount: coins(0, "atom"),
        };
        bank.execute(&api, &mut store, &router, &block, owner, msg)
            .unwrap_err();

        // can mint via sudo
        let msg = BankSudo::Mint {
            to_address: rcpt.to_string(),
            amount: coins(4321, "atom"),
        };
        bank.sudo(&api, &mut store, &router, &block, msg).unwrap();

        // mint fails with 0 tokens
        let msg = BankSudo::Mint {
            to_address: rcpt.to_string(),
            amount: coins(0, "atom"),
        };
        bank.sudo(&api, &mut store, &router, &block, msg)
            .unwrap_err();

        // mint fails with no tokens
        let msg = BankSudo::Mint {
            to_address: rcpt.to_string(),
            amount: vec![],
        };
        bank.sudo(&api, &mut store, &router, &block, msg)
            .unwrap_err();
    }
    */
}
