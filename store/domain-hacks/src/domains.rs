use crate::error::Result;
use crate::types::{DnsStatus, Domain, DomainAvailability, HttpStatus, PhaseOutput};
use crate::utils::{check_dns, check_http, verify_availability};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct GoDaddyDomainResponse {
    available: Option<bool>,
    price: Option<f64>,
    currency: Option<String>,
    period: Option<i32>,
    registration_url: Option<String>,
}

pub struct DomainChecker {
    client: Client,
    api_key: String,
    api_secret: String,
}

impl DomainChecker {
    pub fn new(api_key: String, api_secret: String) -> Self {
        let client = Client::new();
        Self {
            client,
            api_key,
            api_secret,
        }
    }

    pub async fn check_availability(&self, domain: &Domain) -> Result<DomainAvailability> {
        let url = format!(
            "https://api.godaddy.com/v1/domains/available?domain={}",
            domain.full
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, &self.api_secret)
            .send()
            .await
            .map_err(|e| {
                crate::error::DomainError::AvailabilityCheckFailed(format!(
                    "GoDaddy API request failed: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            return Err(crate::error::DomainError::AvailabilityCheckFailed(format!(
                "GoDaddy API returned status: {}",
                response.status()
            )));
        }

        let godaddy_response: GoDaddyDomainResponse = response.json().await.map_err(|e| {
            crate::error::DomainError::AvailabilityCheckFailed(format!(
                "Failed to parse GoDaddy response: {}",
                e
            ))
        })?;

        let api_available = godaddy_response.available.unwrap_or(false);
        let is_exact_match = api_available;

        let verified = verify_availability(domain, api_available).await?;

        let dns_status = check_dns(domain).await?;
        let http_status = check_http(domain).await?;

        let price_estimate = godaddy_response.price;
        let registration_url = godaddy_response.registration_url;

        Ok(DomainAvailability {
            domain: domain.clone(),
            available: verified,
            is_exact_match,
            verified,
            price_estimate,
            registration_url,
            dns_status,
            http_status,
        })
    }

    pub async fn check_multiple(&self, domains: &[Domain]) -> Result<Vec<DomainAvailability>> {
        let mut tasks = Vec::new();

        for domain in domains {
            let checker = self.check_availability(domain);
            tasks.push(checker);
        }

        let results = futures::future::join_all(tasks).await;

        results
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::error::DomainError::AvailabilityCheckFailed(format!(
                    "Batch check failed: {}",
                    e
                ))
            })
    }
}

pub async fn check_domain_availability(
    domain_str: &str,
    api_key: &str,
    api_secret: &str,
) -> Result<DomainAvailability> {
    let domain = Domain::parse(domain_str)?;
    let checker = DomainChecker::new(api_key.to_string(), api_secret.to_string());
    checker.check_availability(&domain).await
}

pub async fn check_domains_batch(
    domains: &[String],
    api_key: &str,
    api_secret: &str,
) -> Result<Vec<DomainAvailability>> {
    let parsed_domains: Result<Vec<Domain>> = domains.iter().map(|d| Domain::parse(d)).collect();

    let domain_list = parsed_domains?;
    let checker = DomainChecker::new(api_key.to_string(), api_secret.to_string());
    checker.check_multiple(&domain_list).await
}

pub fn group_by_availability(
    results: &[DomainAvailability],
) -> HashMap<String, Vec<DomainAvailability>> {
    let mut groups = HashMap::new();

    groups.insert(
        "available".to_string(),
        results
            .iter()
            .filter(|r| r.available && r.verified)
            .cloned()
            .collect(),
    );

    groups.insert(
        "unavailable".to_string(),
        results.iter().filter(|r| !r.available).cloned().collect(),
    );

    groups.insert(
        "unverified".to_string(),
        results
            .iter()
            .filter(|r| r.available && !r.verified)
            .cloned()
            .collect(),
    );

    groups
}

pub fn calculate_total_cost(results: &[DomainAvailability]) -> f64 {
    results
        .iter()
        .filter(|r| r.available && r.verified)
        .filter_map(|r| r.price_estimate)
        .sum()
}
