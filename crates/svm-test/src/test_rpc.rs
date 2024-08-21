use std::collections::BTreeMap;
use std::env::VarError;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use tokio::runtime::Runtime;

use crate::utils::{read_json, read_json_gz, WriteOnDrop};

pub fn test_data_path() -> PathBuf {
    // If `TEST_DATA` has been set, use that path.
    match std::env::var("TEST_DATA") {
        Ok(path) => {
            return path
                .parse()
                .unwrap_or_else(|err| panic!("Failed to parse `TEST_DATA` env; err={err}"))
        }
        Err(std::env::VarError::NotUnicode(raw)) => panic!("Invalid `TEST_DATA` var; raw={raw:?}"),
        Err(std::env::VarError::NotPresent) => {}
    }

    // Else try the default path based on manifest location.
    crate::utils::locate_manifest()
        .unwrap_or_else(|| panic!("`TEST_DATA` not set and failed to locate cargo manifest root"))
        .parent()
        .unwrap()
        .join("test-data")
}

pub fn test_static_data_path() -> PathBuf {
    test_data_path().join("static.json")
}

static STATIC_CACHE: OnceLock<RpcCache> = OnceLock::new();

pub fn get_static_cache() -> &'static RpcCache {
    STATIC_CACHE.get_or_init(|| read_json(&test_static_data_path()))
}

pub struct TestRpc {
    static_cache: &'static RpcCache,
    cache: RwLock<WriteOnDrop<RpcCache>>,
    /// If the RPC is set the cache file will be ignored & overwritten.
    rpc: Option<RpcClient>,
}

impl TestRpc {
    fn unwrap_rpc(&self, key: &Pubkey) -> &RpcClient {
        self.rpc.as_ref().unwrap_or_else(|| {
            panic!("Test tried to access an uncached account and TEST_RPC is not set; key={key:?}")
        })
    }
}

impl TestRpc {
    pub fn load_snapshot(slot: u64) -> Self {
        let static_cache = get_static_cache();
        let cache_path = test_data_path().join(format!("snapshots/{slot}.json.gz"));
        let cache =
            RwLock::new(WriteOnDrop::new(read_json_gz::<RpcCache>(&cache_path), Some(cache_path)));

        TestRpc { static_cache, cache, rpc: None }
    }

    pub fn load_scenario(name: &str) -> Self {
        let cache_path = test_data_path().join(format!("{name}.json.gz"));
        let rpc = match std::env::var("TEST_RPC") {
            Ok(url) => Some(RpcClient::new(url)),
            Err(VarError::NotPresent) => None,
            Err(VarError::NotUnicode(raw)) => panic!("Non utf8 TEST_RPC; raw={raw:?}"),
        };

        assert!(rpc.is_some() || cache_path.exists(), "Need either `TEST_RPC` or test cache file");

        let cache = RwLock::new(WriteOnDrop::new(
            match rpc.is_some() {
                true => RpcCache(BTreeMap::default()),
                false => read_json_gz(&cache_path),
            },
            Some(cache_path),
        ));

        TestRpc { static_cache: get_static_cache(), cache, rpc }
    }

    pub fn account_sync(&self, runtime: &'static Runtime, key: &Pubkey) -> Account {
        runtime.block_on(self.account(key))
    }

    pub async fn account(&self, key: &Pubkey) -> Account {
        // Return cached if exists.
        if let Some(cached) = self
            .static_cache
            .get(key)
            .cloned()
            .or_else(|| self.cache.read().unwrap().get(key).cloned())
        {
            return cached;
        }

        // Load RPC.
        let rpc = self.unwrap_rpc(key);
        let account = rpc
            .get_account_with_config(
                key,
                RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .value
            .unwrap_or_default();

        // Update cache.
        self.cache.write().unwrap().insert(*key, account.clone());

        account
    }
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RpcCache(
    #[serde_as(as = "BTreeMap<serde_with::DisplayFromStr, crate::ser::AccountAsJsonAccount>")]
    pub  BTreeMap<Pubkey, Account>,
);

impl Deref for RpcCache {
    type Target = BTreeMap<Pubkey, Account>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RpcCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
