use crate::domains::{check_domain_availability, check_domains_batch, DomainChecker};
use crate::error::Result;
use crate::strategies::StrategyGenerator;
use crate::types::{Domain, LandingPageTemplate, PhaseOutput};
use async_trait::async_trait;
use lens::{Lens, LensContext, LensError, LensEvent, LensEventStream, LensResult, StreamingLens};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug, Clone, Deserialize)]
pub struct DomainHacksInput {
    #[serde(default)]
    pub domains: DomainsInput,

    pub godaddy_api_key: Option<String>,

    pub godaddy_api_secret: Option<String>,

    #[serde(default)]
    pub generate_strategies: bool,

    #[serde(default)]
    pub generate_landing_pages: bool,

    #[serde(default = "10")]
    pub batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(untagged)]
pub enum DomainsInput {
    #[serde(default)]
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl DomainsInput {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            DomainsInput::None => vec![],
            DomainsInput::Single(d) => vec![d.clone()],
            DomainsInput::Multiple(d) => d.clone(),
        }
    }

    pub fn to_single(&self) -> Option<String> {
        match self {
            DomainsInput::None => None,
            DomainsInput::Single(d) => Some(d.clone()),
            DomainsInput::Multiple(d) => d.first().cloned(),
        }
    }
}

pub struct DomainHacksPlugin {
    id: String,
}

impl DomainHacksPlugin {
    pub fn new() -> Self {
        Self {
            id: "domain-hacks".to_string(),
        }
    }

    fn parse_input(ctx: &LensContext) -> Result<DomainHacksInput> {
        serde_json::from_value(ctx.input.clone())
            .map_err(|e| crate::error::DomainError::Parse(format!("Invalid input: {}", e)))
    }

    fn get_credentials(&self, input: &DomainHacksInput) -> Result<(String, String)> {
        let key = input
            .godaddy_api_key
            .as_ref()
            .or_else(|| std::env::var("GODADDY_API_KEY").ok())
            .ok_or_else(|| {
                crate::error::DomainError::Config("GODADDY_API_KEY not set".to_string())
            })?;

        let secret = input
            .godaddy_api_secret
            .as_ref()
            .or_else(|| std::env::var("GODADDY_API_SECRET").ok())
            .ok_or_else(|| {
                crate::error::DomainError::Config("GODADDY_API_SECRET not set".to_string())
            })?;

        Ok((key.clone(), secret.clone()))
    }

    fn generate_landing_page(
        &self,
        domain: &Domain,
        strategy: &crate::types::DomainStrategy,
    ) -> LandingPageTemplate {
        LandingPageTemplate {
            domain: domain.full.clone(),
            title: format!("{} - Developer Tools", domain.full),
            tagline: "Domain growth hacking for AI-powered development".to_string(),
            description: format!(
                "Discover {} - {} for {}. {}",
                strategy
                    .use_cases
                    .first()
                    .unwrap_or(&"Developer tools".to_string()),
                domain.full,
                strategy.target_audience
            ),
            hero_text: format!(
                "Build and deploy with {} - {}",
                domain.full,
                strategy.tagline()
            ),
            cta_text: "Get Started Free".to_string(),
            features: strategy.use_cases.clone(),
            keywords: strategy.seo_keywords.clone(),
            open_graph: crate::types::OpenGraphMeta {
                title: format!("{} - Developer Tools", domain.full),
                description: format!(
                    "{} for {} - {}. Join thousands of developers using {}.",
                    strategy.tagline(),
                    strategy.target_audience,
                    domain.full
                ),
                image_url: None,
                twitter_card: "summary_large_image".to_string(),
            },
            structured_data: serde_json::to_string(&serde_json::json!({
                "@context": "https://schema.org",
                "@type": "SoftwareApplication",
                "name": format!("{} - Developer Tools", domain.full),
                "description": strategy.tagline(),
                "url": format!("https://{}", domain.full),
                "applicationCategory": "DeveloperApplication",
                "operatingSystem": "Web",
                "offers": {
                    "@type": "Offer",
                    "category": strategy.target_audience,
                    "name": format!("{} - Free Tier", domain.full),
                    "price": "0",
                    "priceCurrency": "USD"
                }
            }))
            .unwrap_or_default(),
        }
    }
}

impl Default for DomainHacksPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Lens for DomainHacksPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Domain Hacks"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn execute(&self, ctx: LensContext) -> std::result::Result<LensResult, LensError> {
        let input =
            Self::parse_input(&ctx).map_err(|e| LensError::ExecutionFailed(e.to_string()))?;

        let domains = input.domains.to_vec();

        if domains.is_empty() {
            return Ok(LensResult::success(serde_json::json!({
                "status": "no_domains",
                "message": "No domains provided. Use streaming mode for full pipeline."
            })));
        }

        let (key, secret) = self
            .get_credentials(&input)
            .map_err(|e| LensError::ExecutionFailed(e.to_string()))?;

        let checker = Arc::new(DomainChecker::new(key, secret));

        let mut results = Vec::new();

        for domain_str in &domains {
            let domain =
                Domain::parse(domain_str).map_err(|e| LensError::ExecutionFailed(e.to_string()))?;

            let checker_for_spawn = checker.clone();
            let result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current()
                    .block_on(checker_for_spawn.check_availability(&domain))
            })
            .await
            .map_err(|e| LensError::ExecutionFailed(e.to_string()))??;

            results.push(result);
        }

        let available_count = results.iter().filter(|r| r.available && r.verified).count();

        Ok(LensResult::success(serde_json::json!({
            "status": "completed",
            "total_domains": results.len(),
            "available": available_count,
            "results": results,
            "message": "Use streaming mode for strategies and landing pages"
        })))
    }
}

#[async_trait]
impl StreamingLens for DomainHacksPlugin {
    async fn execute_streaming(
        &self,
        ctx: LensContext,
    ) -> std::result::Result<(LensResult, LensEventStream), LensError> {
        let (tx, rx) = mpsc::channel(100);
        let start = Instant::now();
        let plugin_id = self.id.clone();

        let input =
            Self::parse_input(&ctx).map_err(|e| LensError::ExecutionFailed(e.to_string()))?;

        let domains = input.domains.to_vec();

        if domains.is_empty() {
            return Ok((
                LensResult::success(serde_json::json!({
                    "status": "no_domains",
                    "message": "No domains provided"
                })),
                Box::pin(ReceiverStream::new(rx)),
            ));
        }

        let (key, secret) = self
            .get_credentials(&input)
            .map_err(|e| LensError::ExecutionFailed(e.to_string()))?;

        let checker = Arc::new(DomainChecker::new(key, secret));

        let domains_for_spawn = domains.clone();
        let generate_strategies = input.generate_strategies;
        let generate_landing_pages = input.generate_landing_pages;

        tokio::spawn(async move {
            let _ = tx
                .send(LensEvent::started(&plugin_id, "domain-growth-hacking"))
                .await;

            let _ = tx
                .send(LensEvent::progress(
                    &plugin_id,
                    "Phase 1: Discovering and parsing domains...",
                ))
                .await;

            let parsed_domains: Vec<Domain> = domains_for_spawn
                .iter()
                .filter_map(|d| Domain::parse(d).ok())
                .collect();

            let _ = tx
                .send(LensEvent::progress(
                    &plugin_id,
                    &format!("Parsed {} domains", parsed_domains.len()),
                ))
                .await;

            let _ = tx
                .send(LensEvent::progress_with_percent(
                    &plugin_id,
                    "Phase 2: Checking domain availability...",
                    25.0,
                ))
                .await;

            let mut availability_results = Vec::new();

            for domain in &parsed_domains {
                let checker_for_spawn = checker.clone();
                let result = tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current()
                        .block_on(checker_for_spawn.check_availability(domain))
                })
                .await;

                match result {
                    Ok(r) => availability_results.push(r),
                    Err(e) => {
                        let _ = tx
                            .send(LensEvent::progress(
                                &plugin_id,
                                &format!("Error checking {}: {}", domain.full, e),
                            ))
                            .await;
                    }
                }
            }

            let available_domains: Vec<_> = availability_results
                .iter()
                .filter(|r| r.available && r.verified)
                .collect();

            let _ = tx
                .send(LensEvent::data(
                    &plugin_id,
                    "availability_check",
                    serde_json::to_value(&availability_results).unwrap_or_default(),
                ))
                .await;

            let _ = tx
                .send(LensEvent::progress(
                    &plugin_id,
                    &format!("{} available domains verified", available_domains.len()),
                ))
                .await;

            if generate_strategies && !available_domains.is_empty() {
                let _ = tx
                    .send(LensEvent::progress_with_percent(
                        &plugin_id,
                        "Phase 3: Generating growth strategies...",
                        50.0,
                    ))
                    .await;

                let generator = StrategyGenerator;
                let mut all_strategies = Vec::new();

                for domain in available_domains {
                    match generator.generate_all(&domain.domain) {
                        Ok(phase_output) => {
                            let _ = tx
                                .send(LensEvent::data(
                                    &plugin_id,
                                    "strategy_generation",
                                    serde_json::to_value(&phase_output).unwrap_or_default(),
                                ))
                                .await;
                            all_strategies.push(phase_output);
                        }
                        Err(e) => {
                            let _ = tx
                                .send(LensEvent::progress(
                                    &plugin_id,
                                    &format!(
                                        "Strategy generation failed for {}: {}",
                                        domain.domain.full, e
                                    ),
                                ))
                                .await;
                        }
                    }
                }
            }

            if generate_landing_pages && !available_domains.is_empty() {
                let _ = tx
                    .send(LensEvent::progress_with_percent(
                        &plugin_id,
                        "Phase 4: Generating landing page templates...",
                        75.0,
                    ))
                    .await;

                if generate_strategies {
                    let generator = StrategyGenerator;

                    for (index, domain) in available_domains.iter().enumerate() {
                        let tier_1 = generator.generate_tier_1(&domain.domain);
                        let tier_2 = generator.generate_tier_2(&domain.domain);
                        let tier_3 = generator.generate_tier_3(&domain.domain);

                        let strategy = tier_1.iter().chain(&tier_2).chain(&tier_3).next();

                        if let Some(ref strategy) = strategy {
                            let template = self.generate_landing_page(&domain.domain, strategy);

                            let _ = tx
                                .send(LensEvent::data(
                                    &plugin_id,
                                    "landing_page",
                                    serde_json::to_value(&template).unwrap_or_default(),
                                ))
                                .await;
                        }
                    }
                } else {
                    for domain in available_domains {
                        let generic_strategy = crate::types::DomainStrategy {
                            domain: domain.domain.full.clone(),
                            tier: crate::types::StrategyTier::Tier1Platform,
                            rationale: "Developer tools and productivity platform".to_string(),
                            target_audience: "Developers, teams, dev shops".to_string(),
                            use_cases: vec![
                                "Tool directory & discovery".to_string(),
                                "Productivity dashboards".to_string(),
                                "Workflow automation".to_string(),
                            ],
                            seo_keywords: vec![
                                "developer tools".to_string(),
                                "productivity platform".to_string(),
                            ],
                            traffic_potential: 5_000_000.0,
                            monetization_paths: vec![
                                "Freemium SaaS".to_string(),
                                "Team plans".to_string(),
                            ],
                        };

                        let template =
                            self.generate_landing_page(&domain.domain, &generic_strategy);

                        let _ = tx
                            .send(LensEvent::data(
                                &plugin_id,
                                "landing_page",
                                serde_json::to_value(&template).unwrap_or_default(),
                            ))
                            .await;
                    }
                }
            }

            let _ = tx
                .send(LensEvent::completed(&plugin_id, start.elapsed()))
                .await;
        });

        let result = LensResult::success(serde_json::json!({
            "status": "streaming",
            "message": "Domain growth hacking pipeline started"
        }));

        Ok((result, Box::pin(ReceiverStream::new(rx))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let plugin = DomainHacksPlugin::new();
        assert_eq!(plugin.id(), "domain-hacks");
        assert_eq!(plugin.name(), "Domain Hacks");
    }

    #[test]
    fn test_parse_input() {
        let ctx = LensContext::new(
            std::env::current_dir().unwrap(),
            serde_json::json!({
                "domains": ["example.com", "test.dev"],
                "generate_strategies": true
            }),
        );

        let input = DomainHacksPlugin::parse_input(&ctx).unwrap();
        assert_eq!(input.domains.to_vec().len(), 2);
        assert_eq!(input.generate_strategies, true);
    }

    #[test]
    fn test_domains_input() {
        let single = DomainsInput::Single("example.com".to_string());
        assert_eq!(single.to_vec().len(), 1);
        assert_eq!(single.to_single().unwrap(), "example.com");

        let multiple = DomainsInput::Multiple(vec!["a.com".to_string(), "b.dev".to_string()]);
        assert_eq!(multiple.to_vec().len(), 2);
        assert_eq!(multiple.to_single().unwrap(), "a.com");

        let none = DomainsInput::None;
        assert!(none.to_vec().is_empty());
        assert!(none.to_single().is_none());
    }
}
