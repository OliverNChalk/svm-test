use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, Weak};

use dashmap::DashMap;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use test_rpc::TestRpc;

use super::*;

static HARNESS: OnceLock<Harness> = OnceLock::new();

pub struct Harness {
    runtime: tokio::runtime::Runtime,
    /// Scenarios represent a shared state cache used by N tests.
    ///
    /// The map holds a `Weak` reference to the `Scenario` to ensure it gets
    /// dropped before the test binary terminates. Unfortunately destructors
    /// do not run on static objects (and Harness is static).
    scenarios: DashMap<&'static str, Weak<Scenario>>,
}

impl Harness {
    pub fn get() -> &'static Harness {
        HARNESS.get_or_init(|| {
            if std::env::var("TEST_DEBUG").is_ok() {
                #[rustfmt::skip]
                solana_logger::setup_with_default(
                    "svm_test=debug,\
                     solana_rbpf::vm=debug,\
                     solana_runtime::message_processor=debug,\
                     solana_runtime::system_instruction_processor=trace",
                );
            }

            Harness {
                runtime: tokio::runtime::Runtime::new().unwrap(),
                scenarios: DashMap::default(),
            }
        })
    }

    pub fn get_scenario(&'static self, name: &'static str) -> Arc<Scenario> {
        let scenario = match self.scenarios.entry(name) {
            dashmap::Entry::Occupied(mut entry) => entry.get().upgrade().unwrap_or_else(|| {
                let scenario = self.load_scenario(name);

                entry.insert(Arc::downgrade(&scenario));

                scenario
            }),
            dashmap::Entry::Vacant(entry) => {
                let scenario = self.load_scenario(name);

                entry.insert(Arc::downgrade(&scenario));

                scenario
            }
        };

        scenario
    }

    pub fn get_snapshot(&'static self, block: u64) -> Arc<Scenario> {
        let rpc = TestRpc::load_snapshot(block);

        Arc::new(Scenario { runtime: &self.runtime, rpc })
    }

    fn load_scenario(&'static self, name: &str) -> Arc<Scenario> {
        let rpc = TestRpc::load_scenario(name);

        Arc::new(Scenario { runtime: &self.runtime, rpc })
    }
}

#[derive(Debug)]
pub struct Scenario {
    runtime: &'static tokio::runtime::Runtime,
    rpc: TestRpc,
}

impl Scenario {
    pub fn with_overrides(
        self: &Arc<Self>,
        overrides: BTreeMap<Pubkey, Account>,
    ) -> ScenarioWithOverrides {
        ScenarioWithOverrides { scenario: self.clone(), overrides }
    }
}

#[async_trait::async_trait]
impl FetchAccounts for Scenario {
    async fn account(&self, key: &Pubkey) -> Account {
        self.rpc.account(key).await
    }
}

impl AccountLoader for Scenario {
    fn load(&self, key: &Pubkey) -> Account {
        self.rpc.account_sync(self.runtime, key)
    }
}

pub struct ScenarioWithOverrides {
    scenario: Arc<Scenario>,
    pub overrides: BTreeMap<Pubkey, Account>,
}

impl ScenarioWithOverrides {
    pub fn runtime(&self) -> &'static tokio::runtime::Runtime {
        self.scenario.runtime
    }
}

#[async_trait::async_trait]
impl FetchAccounts for ScenarioWithOverrides {
    async fn account(&self, key: &Pubkey) -> Account {
        if let Some(cached) = self.overrides.get(key) {
            return cached.to_owned();
        }

        self.scenario.account(key).await
    }
}

impl AccountLoader for ScenarioWithOverrides {
    fn load(&self, key: &Pubkey) -> Account {
        if let Some(cached) = self.overrides.get(key) {
            return cached.to_owned();
        }

        self.scenario.load(key)
    }
}
