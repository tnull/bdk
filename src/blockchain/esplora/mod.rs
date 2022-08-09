//! Esplora
//!
//! This module defines a [`EsploraBlockchain`] struct that can query an Esplora
//! backend populate the wallet's [database](crate::database::Database) by:
//!
//! ## Example
//!
//! ```no_run
//! # use bdk::blockchain::esplora::EsploraBlockchain;
//! let blockchain = EsploraBlockchain::new("https://blockstream.info/testnet/api", 20);
//! # Ok::<(), bdk::Error>(())
//! ```
//!
//! Esplora blockchain can use either `ureq` or `reqwest` for the HTTP client
//! depending on your needs (blocking or async respectively).
//!
//! Please note, to configure the Esplora HTTP client correctly use one of:
//! Blocking:  --features='esplora,ureq'
//! Async:     --features='async-interface,esplora,reqwest' --no-default-features

pub use esplora_client::Error as EsploraError;

#[cfg(feature = "use-esplora-reqwest")]
mod reqwest;

#[cfg(feature = "use-esplora-reqwest")]
pub use self::reqwest::*;

#[cfg(feature = "use-esplora-ureq")]
mod ureq;

#[cfg(feature = "use-esplora-ureq")]
pub use self::ureq::*;

/// Configuration for an [`EsploraBlockchain`]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct EsploraBlockchainConfig {
    /// Base URL of the esplora service
    ///
    /// eg. `https://blockstream.info/api/`
    pub base_url: String,
    /// Optional URL of the proxy to use to make requests to the Esplora server
    ///
    /// The string should be formatted as: `<protocol>://<user>:<password>@host:<port>`.
    ///
    /// Note that the format of this value and the supported protocols change slightly between the
    /// sync version of esplora (using `ureq`) and the async version (using `reqwest`). For more
    /// details check with the documentation of the two crates. Both of them are compiled with
    /// the `socks` feature enabled.
    ///
    /// The proxy is ignored when targeting `wasm32`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Number of parallel requests sent to the esplora service (default: 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u8>,
    /// Stop searching addresses for transactions after finding an unused gap of this length.
    pub stop_gap: usize,
    /// Socket timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl EsploraBlockchainConfig {
    /// create a config with default values given the base url and stop gap
    pub fn new(base_url: String, stop_gap: usize) -> Self {
        Self {
            base_url,
            proxy: None,
            timeout: None,
            stop_gap,
            concurrency: None,
        }
    }
}

impl From<esplora_client::BlockTime> for crate::BlockTime {
    fn from(esplora_client::BlockTime { timestamp, height }: esplora_client::BlockTime) -> Self {
        Self { timestamp, height }
    }
}

#[cfg(test)]
#[cfg(feature = "test-esplora")]
crate::bdk_blockchain_tests! {
    fn test_instance(test_client: &TestClient) -> EsploraBlockchain {
        EsploraBlockchain::new(&format!("http://{}",test_client.electrsd.esplora_url.as_ref().unwrap()), 20)
    }
}

const DEFAULT_CONCURRENT_REQUESTS: u8 = 4;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn feerate_parsing() {
        let esplora_fees = serde_json::from_str::<HashMap<String, f64>>(
            r#"{
  "25": 1.015,
  "5": 2.3280000000000003,
  "12": 2.0109999999999997,
  "15": 1.018,
  "17": 1.018,
  "11": 2.0109999999999997,
  "3": 3.01,
  "2": 4.9830000000000005,
  "6": 2.2359999999999998,
  "21": 1.018,
  "13": 1.081,
  "7": 2.2359999999999998,
  "8": 2.2359999999999998,
  "16": 1.018,
  "20": 1.018,
  "22": 1.017,
  "23": 1.017,
  "504": 1,
  "9": 2.2359999999999998,
  "14": 1.018,
  "10": 2.0109999999999997,
  "24": 1.017,
  "1008": 1,
  "1": 4.9830000000000005,
  "4": 2.3280000000000003,
  "19": 1.018,
  "144": 1,
  "18": 1.018
}
"#,
        )
        .unwrap();
        assert_eq!(
            into_fee_rate(6, esplora_fees.clone()).unwrap(),
            FeeRate::from_sat_per_vb(2.236)
        );
        assert_eq!(
            into_fee_rate(26, esplora_fees).unwrap(),
            FeeRate::from_sat_per_vb(1.015),
            "should inherit from value for 25"
        );
    }

    #[test]
    #[cfg(feature = "test-esplora")]
    fn test_esplora_with_variable_configs() {
        use crate::testutils::{
            blockchain_tests::TestClient,
            configurable_blockchain_tests::ConfigurableBlockchainTester,
        };

        struct EsploraTester;

        impl ConfigurableBlockchainTester<EsploraBlockchain> for EsploraTester {
            const BLOCKCHAIN_NAME: &'static str = "Esplora";

            fn config_with_stop_gap(
                &self,
                test_client: &mut TestClient,
                stop_gap: usize,
            ) -> Option<EsploraBlockchainConfig> {
                Some(EsploraBlockchainConfig {
                    base_url: format!(
                        "http://{}",
                        test_client.electrsd.esplora_url.as_ref().unwrap()
                    ),
                    proxy: None,
                    concurrency: None,
                    stop_gap: stop_gap,
                    timeout: None,
                })
            }
        }

        EsploraTester.run();
    }
}
