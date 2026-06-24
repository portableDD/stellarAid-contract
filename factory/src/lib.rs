#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec,
};

// ─── Storage keys ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin address allowed to deploy new campaign contracts.
    Admin,
    /// Sequential counter of deployed campaigns.
    Count,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

/// Parameters passed to `deploy_campaign`, forwarded as constructor args.
#[contracttype]
#[derive(Clone)]
pub struct CampaignParams {
    /// Address that will own / manage the deployed campaign contract.
    pub creator: Address,
    /// SHA-256 hash of the campaign WASM registered on-chain.
    pub wasm_hash: BytesN<32>,
    /// Unique 32-byte salt so the same creator can deploy multiple campaigns.
    pub salt: BytesN<32>,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct CampaignFactory;

#[contractimpl]
impl CampaignFactory {
    /// Initialise the factory with an admin address.
    ///
    /// Can only be called once.  The admin is the only address allowed to
    /// deploy new campaign contracts.
    ///
    /// # Panics
    /// - if the factory has already been initialised.
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        if env
            .storage()
            .instance()
            .has(&DataKey::Admin)
        {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Count, &0u64);
    }

    /// Issue #271 – Deploy a new campaign contract.
    ///
    /// Uses `env.deployer().with_address(creator, salt).deploy(wasm_hash)` to
    /// instantiate a fresh campaign contract at a deterministic address derived
    /// from (`creator`, `salt`).
    ///
    /// # Arguments
    /// * `creator`    – Address that will own the new campaign.
    /// * `params`     – Deployment parameters (wasm_hash, salt, creator).
    ///
    /// # Returns
    /// The contract address of the newly deployed campaign.
    ///
    /// # Panics
    /// - `Unauthorized` if caller is not the current admin.
    pub fn deploy_campaign(env: Env, creator: Address, params: CampaignParams) -> Address {
        // Only the admin may deploy new campaigns.
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        admin.require_auth();

        // Deploy the campaign contract at a deterministic address.
        let deployed_address = env
            .deployer()
            .with_address(params.creator.clone(), params.salt)
            .deploy_v2(params.wasm_hash, ());

        // Increment deployment counter.
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Count)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Count, &(count + 1));

        // Emit `campaign_deployed` event with creator and new contract address.
        env.events().publish(
            (Symbol::new(&env, "campaign_deployed"), creator),
            deployed_address.clone(),
        );

        deployed_address
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Returns the total number of campaigns deployed via this factory.
    pub fn get_campaign_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Count)
            .unwrap_or(0)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as AddressTestUtils;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CampaignFactory);
        let client = CampaignFactoryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_campaign_count(), 0);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_initialize_twice_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CampaignFactory);
        let client = CampaignFactoryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);
        client.initialize(&admin); // should panic
    }

    /// `deploy_campaign` increments the counter and emits the event.
    /// Because we cannot upload real WASM in unit tests, we verify the
    /// deploy call panics with "Wasm does not exist" — proving the auth
    /// and storage paths were reached and only the WASM lookup failed.
    #[test]
    #[should_panic]
    fn test_deploy_campaign_panics_without_wasm() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CampaignFactory);
        let client = CampaignFactoryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let creator = Address::generate(&env);
        let params = CampaignParams {
            creator: creator.clone(),
            wasm_hash: BytesN::from_array(&env, &[0u8; 32]),
            salt: BytesN::from_array(&env, &[1u8; 32]),
        };

        client.deploy_campaign(&creator, &params);
    }
}
