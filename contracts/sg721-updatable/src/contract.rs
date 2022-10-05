#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use sg721_base::msg::CollectionInfoResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::FROZEN_TOKEN_METADATA;
use sg721::InstantiateMsg;

use cw721_base::Extension;
use cw_utils::nonpayable;
use sg721_base::ContractError::Unauthorized;
use sg721_base::Sg721Contract;
pub type Sg721UpdatableContract<'a> = Sg721Contract<'a, Extension>;
use sg_std::Response;

const CONTRACT_NAME: &str = "crates.io:sg721-updatable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set frozen to false on instantiate. allows updating token metadata
    FROZEN_TOKEN_METADATA.save(deps.storage, &false)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let res = Sg721UpdatableContract::default().instantiate(deps, env, info, msg)?;
    Ok(res
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FreezeTokenMetadata {} => execute_freeze_token_metadata(deps, env, info),
        // ExecuteMsg::UpdateTokenMetadata {
        //     token_id,
        //     token_uri,
        // } => execute_update_token_metadata(deps, env, info, token_id, token_uri),
        // add other msgs here
        // _ => Sg721UpdatableContract::default().execute(deps, env, info,),
    }

    // update token metadata
    // freeze token metadata
}

pub fn execute_freeze_token_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check if sender is creator
    let owner = deps.api.addr_validate(&info.sender.to_string())?;
    let collection_info: CollectionInfoResponse =
        Sg721UpdatableContract::default().query_collection_info(deps.as_ref())?;

    if owner != collection_info.creator {
        return Err(ContractError::Base(Unauthorized {}));
    }

    FROZEN_TOKEN_METADATA.save(deps.storage, &true)?;

    Ok(Response::new()
        .add_attribute("action", "freeze_token_metadata")
        .add_attribute("frozen", "true"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    // all the same but add query freeze status
    unimplemented!()
}