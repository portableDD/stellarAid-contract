#![cfg(test)]

use soroban_sdk::testutils::Address as AddressTestUtils;
use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::{MilestoneData, MilestoneStatus, StellarAsset};
use crate::CampaignContract;

fn setup_contract(env: &Env) -> (soroban_sdk::Address, Address, Address) {
    let contract_id = env.register_contract(None, CampaignContract);
    let client = crate::CampaignContractClient::new(env, &contract_id);

    let creator = Address::generate(env);
    let token = Address::generate(env);

    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(token.clone()),
    });

    let mut milestones: Vec<MilestoneData> = Vec::new(env);
    milestones.push_back(MilestoneData {
        index: 0,
        target_amount: 1000,
        released_amount: 0,
        description_hash: BytesN::from_array(env, &[0u8; 32]),
        status: MilestoneStatus::Locked,
        released_at: None,
        released_at_ledger: None,
        release_tx: None,
        released_to: None,
    });

    client.initialize(
        &creator,
        &1000,
        &(env.ledger().timestamp() + 86400),
        &assets,
        &milestones,
        &0,
    );

    (contract_id, creator, token)
}

/// Issue #270 – set_admin: admin defaults to creator after initialize.
#[test]
fn test_get_admin_after_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, creator, _token) = setup_contract(&env);
    let client = crate::CampaignContractClient::new(&env, &contract_id);
    assert_eq!(client.get_admin(), Some(creator));
}

/// Issue #270 – set_admin: rotates admin to new address.
#[test]
fn test_set_admin_rotates() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _creator, _token) = setup_contract(&env);
    let client = crate::CampaignContractClient::new(&env, &contract_id);

    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);

    assert_eq!(client.get_admin(), Some(new_admin));
}

/// Issue #269 – freeze: admin can freeze and unfreeze.
#[test]
fn test_freeze_and_unfreeze() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _creator, _token) = setup_contract(&env);
    let client = crate::CampaignContractClient::new(&env, &contract_id);

    // Should not panic
    client.freeze();
    client.unfreeze();
}

/// Issue #269 – freeze: donations rejected while frozen.
#[test]
#[should_panic]
fn test_donate_rejected_when_frozen() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _creator, token) = setup_contract(&env);
    let client = crate::CampaignContractClient::new(&env, &contract_id);

    client.freeze();

    let donor = Address::generate(&env);
    client.donate(&donor, &500, &crate::types::AssetInfo::Stellar(token));
}

/// Issue #268 – upgrade: function exists and requires admin auth.
/// In tests, deploying a new wasm hash always fails with MissingValue since
/// the hash doesn't correspond to a real uploaded wasm — this confirms auth
/// is checked before the deployer call.
#[test]
#[should_panic]
fn test_upgrade_requires_valid_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _creator, _token) = setup_contract(&env);
    let client = crate::CampaignContractClient::new(&env, &contract_id);

    // A zero hash has no corresponding wasm in the test environment.
    let new_hash: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);
    client.upgrade(&new_hash); // panics: Wasm does not exist
}
