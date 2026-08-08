use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct MetricProfile {
    pub profile_id: String,
    pub vertical: String,
    pub schema_version: String,
    pub supported_algorithms: Vec<String>,
    pub max_latency_ms: u64,
    pub min_quality_score: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("profile already exists")]
    DuplicateProfile,
    #[error("invalid profile")]
    InvalidProfile,
    #[error("profile not found")]
    ProfileNotFound,
}

pub struct MetricProfileRegistry {
    profiles: HashMap<String, MetricProfile>,
}

impl MetricProfileRegistry {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn register_profile(&mut self, profile: MetricProfile) -> Result<(), RegistryError> {
        if self.profiles.contains_key(&profile.profile_id) {
            return Err(RegistryError::DuplicateProfile);
        }
        if profile.profile_id.is_empty()
            || profile.vertical.is_empty()
            || profile.schema_version.is_empty()
            || profile.supported_algorithms.is_empty()
            || !(0.0..=1.0).contains(&profile.min_quality_score)
        {
            return Err(RegistryError::InvalidProfile);
        }

        self.profiles.insert(profile.profile_id.clone(), profile);
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<&MetricProfile, RegistryError> {
        self.profiles.get(profile_id).ok_or(RegistryError::ProfileNotFound)
    }

    pub fn list_profiles(&self) -> Vec<&MetricProfile> {
        let mut out = self.profiles.values().collect::<Vec<_>>();
        out.sort_by(|a, b| a.profile_id.cmp(&b.profile_id));
        out
    }

    pub fn compatible_profile_pairs(&self) -> Vec<(String, String)> {
        let profiles = self.list_profiles();
        let mut out = Vec::new();
        for i in 0..profiles.len() {
            for j in (i + 1)..profiles.len() {
                let a = profiles[i];
                let b = profiles[j];
                if are_profiles_compatible(a, b) {
                    out.push((a.profile_id.clone(), b.profile_id.clone()));
                }
            }
        }
        out
    }
}

impl Default for MetricProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn are_profiles_compatible(a: &MetricProfile, b: &MetricProfile) -> bool {
    same_major_version(&a.schema_version, &b.schema_version)
        && a.supported_algorithms
            .iter()
            .any(|algo| b.supported_algorithms.iter().any(|other| other == algo))
}

fn same_major_version(a: &str, b: &str) -> bool {
    let major_a = a.split('.').next().unwrap_or_default();
    let major_b = b.split('.').next().unwrap_or_default();
    !major_a.is_empty() && major_a == major_b
}

#[cfg(test)]
mod tests {
    use super::{are_profiles_compatible, MetricProfile, MetricProfileRegistry, RegistryError};

    fn profile(
        id: &str,
        vertical: &str,
        schema: &str,
        algorithms: &[&str],
        quality: f64,
    ) -> MetricProfile {
        MetricProfile {
            profile_id: id.to_string(),
            vertical: vertical.to_string(),
            schema_version: schema.to_string(),
            supported_algorithms: algorithms.iter().map(|value| value.to_string()).collect(),
            max_latency_ms: 1_000,
            min_quality_score: quality,
        }
    }

    #[test]
    fn registry_accepts_multi_vertical_profiles() {
        let mut registry = MetricProfileRegistry::new();
        registry
            .register_profile(profile(
                "industrial-v1",
                "industrial_iot",
                "1.0.0",
                &["viterbi-basic-bsc", "ldpc-minsum-basic"],
                0.7,
            ))
            .expect("register industrial");
        registry
            .register_profile(profile(
                "satellite-v1",
                "space_satellite",
                "1.1.0",
                &["ldpc-minsum-basic", "turbo-basic"],
                0.75,
            ))
            .expect("register satellite");

        let profiles = registry.list_profiles();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn compatibility_requires_same_major_and_algorithm_overlap() {
        let a = profile("a", "industrial", "1.0.0", &["ldpc-minsum-basic"], 0.7);
        let b = profile("b", "space", "1.2.0", &["ldpc-minsum-basic"], 0.7);
        let c = profile("c", "space", "2.0.0", &["ldpc-minsum-basic"], 0.7);
        let d = profile("d", "audio", "1.0.0", &["turbo-basic"], 0.7);

        assert!(are_profiles_compatible(&a, &b));
        assert!(!are_profiles_compatible(&a, &c));
        assert!(!are_profiles_compatible(&a, &d));
    }

    #[test]
    fn duplicate_profile_is_rejected() {
        let mut registry = MetricProfileRegistry::new();
        let p = profile("industrial-v1", "industrial", "1.0.0", &["viterbi-basic-bsc"], 0.7);
        registry.register_profile(p.clone()).expect("first insert ok");
        let err = registry.register_profile(p).expect_err("duplicate should fail");
        assert_eq!(err, RegistryError::DuplicateProfile);
    }
}
