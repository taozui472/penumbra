use std::{fmt::Display, sync::Arc};

use super::substore::SubstoreConfig;

/// A collection of substore, each with a unique prefix.
#[derive(Debug, Clone)]
pub struct MultistoreConfig {
    pub main_store: Arc<SubstoreConfig>,
    pub substores: Vec<Arc<SubstoreConfig>>,
}

impl MultistoreConfig {
    pub fn iter(&self) -> impl Iterator<Item = &Arc<SubstoreConfig>> {
        self.substores.iter()
    }

    /// Returns the substore matching the key's prefix, return `None` otherwise.
    pub fn find_substore(&self, key: &[u8]) -> Arc<SubstoreConfig> {
        let key = key.as_ref();
        // Note: This is a linear search, but the number of substores is small.
        self.substores
            .iter()
            .find(|s| key.starts_with(s.prefix.as_bytes()))
            .cloned()
            .unwrap_or(self.main_store.clone())
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_str<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        let config = self.find_substore(key.as_bytes());
        if key == config.prefix {
            return (key, self.main_store.clone());
        }

        let key = key
            .strip_prefix(&config.prefix)
            .expect("key has the prefix of the matched substore");
        (key, config)
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_bytes<'a>(&self, key: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        let config = self.find_substore(key);
        if key == config.prefix.as_bytes() {
            return (key, self.main_store.clone());
        }

        let key = key
            .strip_prefix(config.prefix.as_bytes())
            .expect("key has the prefix of the matched substore");

        (key, config)
    }
}

impl Default for MultistoreConfig {
    fn default() -> Self {
        Self {
            main_store: Arc::new(SubstoreConfig::new("")),
            substores: vec![],
        }
    }
}

/// Tracks the latest version of each substore, and wraps a `MultistoreConfig`.
#[derive(Debug)]
pub struct MultistoreCache {
    pub config: MultistoreConfig,
    pub substores: std::collections::BTreeMap<Arc<SubstoreConfig>, jmt::Version>,
}

impl Default for MultistoreCache {
    fn default() -> Self {
        Self {
            config: MultistoreConfig::default(),
            substores: std::collections::BTreeMap::new(),
        }
    }
}

impl Display for MultistoreCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for (substore, version) in &self.substores {
            s.push_str(&format!("{}: {}\n", substore.prefix, version));
        }
        write!(f, "{}", s)
    }
}

impl MultistoreCache {
    pub fn from_config(config: MultistoreConfig) -> Self {
        Self {
            config,
            substores: std::collections::BTreeMap::new(),
        }
    }

    pub fn set_version(&mut self, substore: Arc<SubstoreConfig>, version: jmt::Version) {
        self.substores.insert(substore, version);
    }

    pub fn get_version(&self, substore: &Arc<SubstoreConfig>) -> Option<jmt::Version> {
        self.substores.get(substore).cloned()
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_str<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        self.config.route_key_str(key)
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_bytes<'a>(&self, key: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        self.config.route_key_bytes(key)
    }
}