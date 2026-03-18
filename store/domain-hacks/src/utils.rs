use crate::error::Result;
use crate::types::{DnsStatus, Domain, HttpStatus};
use reqwest::Client;
use std::time::Duration;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

pub async fn check_dns(domain: &Domain) -> Result<DnsStatus> {
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    )
    .map_err(|e| crate::error::DomainError::DnsResolutionFailed(
        format!("Failed to create resolver: {}", e)
    ))?;

    let full_domain = &domain.full;

    let has_a_record = match resolver.ipv4_lookup(full_domain).await {
        Ok(lookup) => !lookup.iter().collect::<Vec<_>>().is_empty(),
        Err(_) => false,
    };

    let has_mx_record = match resolver.mx_lookup(full_domain).await {
        Ok(lookup) => !lookup.iter().collect::<Vec<_>>().is_empty(),
        Err(_) => false,
    };

    let has_ns_records = match resolver.ns_lookup(full_domain).await {
        Ok(lookup) => !lookup.iter().collect::<Vec<_>>().is_empty(),
        Err(_) => false,
    };

    let resolvable = has_a_record || has_ns_records;

    Ok(DnsStatus {
        has_a_record,
        has_mx_record,
        has_ns_records,
        resolvable,
    })
}

pub async fn check_http(domain: &Domain) -> Result<HttpStatus> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(2))
        .build()
        .map_err(|e| crate::error::DomainError::HttpVerificationFailed(
            format!("Failed to create HTTP client: {}", e)
        ))?;

    let url = format!("https://{}", domain.full);

    match client.get(&url).send().await {
        Ok(response) => {
            let server_type = response
                .headers()
                .get("server")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            Ok(HttpStatus {
                accessible: true,
                status_code: Some(response.status().as_u16()),
                server_type,
                content_type,
            })
        }
        Err(e) => {
            if e.is_connect() || e.is_timeout() {
                Ok(HttpStatus {
                    accessible: false,
                    status_code: None,
                    server_type: None,
                    content_type: None,
                })
            } else {
                Err(crate::error::DomainError::HttpVerificationFailed(
                    format!("HTTP request failed: {}", e)
                ))
            }
        }
    }
}

pub async fn verify_availability(domain: &Domain, api_available: bool) -> Result<bool> {
    if !api_available {
        return Ok(false);
    }

    let dns_status = check_dns(domain).await?;
    let http_status = check_http(domain).await?;

    let actually_available = !dns_status.resolvable && !http_status.accessible;

    Ok(actually_available)
}

pub fn extract_domain_from_url(url: &str) -> Result<Domain> {
    let url = url.trim().trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("www.");

    let domain_part = url.split('/').next()
        .ok_or_else(|| crate::error::DomainError::InvalidDomain(
            format!("Invalid URL: {}", url)
        ))?;

    Domain::parse(domain_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_parse() {
        let domain = Domain::parse("example.com").unwrap();
        assert_eq!(domain.name, "example");
        assert_eq!(domain.tld, "com");
        assert_eq!(domain.full, "example.com");
    }

    #[test]
    fn test_extract_domain() {
        let domain = extract_domain_from_url("https://example.com/path").unwrap();
        assert_eq!(domain.full, "example.com");

        let domain = extract_domain_from_url("http://www.example.com").unwrap();
        assert_eq!(domain.full, "example.com");
    }
}
