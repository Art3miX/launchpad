use async_std::task;
use cosmwasm_std::{Addr, Attribute};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::H160;

use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper};

// use sg_std::StargazeMsgWrapper;
use std::str;

use crate::contract::reply;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer, Wallet, WalletError};
use eyre::Result;

extern crate whitelist_generic;

const OWNER: &str = "admin0001";
const AIRDROP_CONTRACT: &str = "contract0";
const STARGAZE_WALLET_01: &str = "0xstargaze_wallet_01";
const CONTRACT_CONFIG_PLAINTEXT: &str = "My Stargaze address is {wallet} and I want a Winter Pal.";

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract() -> Box<dyn Contract<sg_std::StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(reply);
    Box::new(contract)
}

pub fn whitelist_generic_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_generic::contract::execute,
        whitelist_generic::contract::instantiate,
        whitelist_generic::contract::query,
    );
    Box::new(contract)
}

fn get_instantiate_contract(addresses: Vec<String>) -> StargazeApp {
    let mut app = custom_mock_app();
    let sg_eth_id = app.store_code(contract());
    let whitelist_code_id = app.store_code(whitelist_generic_contract());
    assert_eq!(sg_eth_id, 1);
    let msg: InstantiateMsg = InstantiateMsg {
        admin: Addr::unchecked(OWNER),
        claim_msg_plaintext: Addr::unchecked(CONTRACT_CONFIG_PLAINTEXT).into_string(),
        amount: 3000,
        minter_page: "http://levana_page/airdrop".to_string(),
        addresses,
        minter_code_id: whitelist_code_id,
    };
    app.instantiate_contract(
        sg_eth_id,
        Addr::unchecked(OWNER),
        &msg,
        &[],
        "sg-eg-airdrop",
        Some(Addr::unchecked(OWNER).to_string()),
    )
    .unwrap();
    app
}

async fn get_signature(
    wallet: Wallet<SigningKey>,
    plaintext_msg: &str,
) -> Result<ethers_core::types::Signature, WalletError> {
    wallet.sign_message(plaintext_msg).await
}

fn get_wallet_and_sig(
    claim_plaintext: String,
) -> (
    Wallet<ethers_core::k256::ecdsa::SigningKey>,
    std::string::String,
    H160,
    std::string::String,
) {
    let wallet = LocalWallet::new(&mut thread_rng());
    let eth_sig_str = task::block_on(get_signature(wallet.clone(), &claim_plaintext))
        .unwrap()
        .to_string();
    let eth_address = wallet.address();
    let eth_addr_str = format!("{:?}", eth_address);
    (wallet, eth_sig_str, eth_address, eth_addr_str)
}

fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
) -> Result<AppResponse, Error> {
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let result = app.execute_contract(user, sg_eth_addr, &msg, &[]).unwrap();
    Ok(result)
}

fn get_msg_plaintext(wallet_address: String) -> String {
    str::replace(CONTRACT_CONFIG_PLAINTEXT, "{wallet}", &wallet_address)
}

#[test]
fn test_instantiate() {
    get_instantiate_contract(vec![]);
}

#[test]
fn test_instantiate_with_addresses() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];
    let app = get_instantiate_contract(addresses);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "addr1".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr.clone(), &query_msg)
        .unwrap();
    assert!(result);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "lies".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr, &query_msg)
        .unwrap();
    assert!(!result);
}

#[test]
fn test_not_authorized_add_eth() {
    let mut app = get_instantiate_contract(vec![]);

    let fake_admin = Addr::unchecked("fake_admin");
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let res = app.execute_contract(fake_admin, sg_eth_addr, &execute_msg, &[]);
    let error = res.unwrap_err();
    let expected_err_msg = "Unauthorized admin, sender is fake_admin";
    assert_eq!(error.root_cause().to_string(), expected_err_msg)
}

#[test]
fn test_authorized_add_eth() {
    let mut app = get_instantiate_contract(vec![]);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let owner_admin = Addr::unchecked(OWNER);
    let res = app.execute_contract(owner_admin, sg_eth_addr, &execute_msg, &[]);
    res.unwrap();
}

#[test]
fn test_add_eth_and_verify() {
    let mut app = get_instantiate_contract(vec![]);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let eth_address_str = Addr::unchecked("testing_addr").to_string();
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address_str.clone()],
    };

    // test before add:
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address_str.clone(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr.clone(), &query_msg)
        .unwrap();
    assert!(!result);

    let owner_admin = Addr::unchecked(OWNER);
    let _ = app.execute_contract(owner_admin, sg_eth_addr.clone(), &execute_msg, &[]);

    //test after add
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address_str,
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr, &query_msg)
        .unwrap();
    assert!(result);
}

#[test]
fn test_valid_eth_sig_claim() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = get_instantiate_contract(vec![eth_addr_str.clone()]);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let res = execute_contract_with_msg(claim_message, &mut app, stargaze_wallet_01).unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "3000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "is_eligible".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}

#[test]
fn test_invalid_eth_sig_claim() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let (_, eth_sig_str_2, _, _) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = get_instantiate_contract(vec![eth_addr_str.clone()]);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str_2,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let res = execute_contract_with_msg(claim_message, &mut app, stargaze_wallet_01).unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "3000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "is_eligible".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}

#[test]
fn test_can_not_claim_twice() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = get_instantiate_contract(vec![eth_addr_str.clone()]);
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let res =
        execute_contract_with_msg(claim_message.clone(), &mut app, stargaze_wallet_01.clone())
            .unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "3000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "is_eligible".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);

    let res = execute_contract_with_msg(claim_message, &mut app, stargaze_wallet_01).unwrap();
    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "3000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "is_eligible".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}
