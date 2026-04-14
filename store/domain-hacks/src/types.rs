use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub tld: String,
    pub full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAvailability {
    pub domain: Domain,
    pub available: bool,
    pub is_exact_match: bool,
    pub verified: bool,
    pub price_estimate: Option<f64>,
    pub registration_url: Option<String>,
    pub dns_status: DnsStatus,
    pub http_status: HttpStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsStatus {
    pub has_a_record: bool,
    pub has_mx_record: bool,
    pub has_ns_records: bool,
    pub resolvable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpStatus {
    pub accessible: bool,
    pub status_code: Option<u16>,
    pub server_type: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyTier {
    #[serde(rename = "tier_1_platform")]
    Tier1Platform,
    #[serde(rename = "tier_2_action")]
    Tier2Action,
    #[serde(rename = "tier_3_category")]
    Tier3Category,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStrategy {
    pub domain: String,
    pub tier: StrategyTier,
    pub rationale: String,
    pub target_audience: String,
    pub use_cases: Vec<String>,
    pub seo_keywords: Vec<String>,
    pub traffic_potential: f64,
    pub monetization_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendations {
    pub tier_1_platforms: Vec<DomainStrategy>,
    pub tier_2_actions: Vec<DomainStrategy>,
    pub tier_3_categories: Vec<DomainStrategy>,
    pub total_cost_estimate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingPageTemplate {
    pub domain: String,
    pub title: String,
    pub tagline: String,
    pub description: String,
    pub hero_text: String,
    pub cta_text: String,
    pub features: Vec<String>,
    pub keywords: Vec<String>,
    pub open_graph: OpenGraphMeta,
    pub structured_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGraphMeta {
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub twitter_card: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoRecommendations {
    pub domain: String,
    pub title_suggestions: Vec<String>,
    pub meta_description: String,
    pub keywords: Vec<String>,
    pub content_suggestions: Vec<String>,
    pub backlink_targets: Vec<String>,
    pub social_media_handles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PhaseOutput {
    #[serde(rename = "availability_check")]
    AvailabilityCheck { results: Vec<DomainAvailability> },
    #[serde(rename = "strategy_generation")]
    StrategyGeneration { strategies: StrategyRecommendations },
    #[serde(rename = "landing_page")]
    LandingPage { template: LandingPageTemplate },
    #[serde(rename = "seo_recommendations")]
    SeoRecommendations { seo: SeoRecommendations },
}

impl Domain {
    pub fn parse(domain: &str) -> crate::Result<Self> {
        let domain = domain.trim().to_lowercase();
        let parts: Vec<&str> = domain.split('.').collect();

        if parts.len() < 2 {
            return Err(crate::error::DomainError::InvalidDomain(format!(
                "Invalid domain: {}",
                domain
            )));
        }

        let tld = parts.last().unwrap().to_string();
        let name = parts[..parts.len() - 1].join(".");

        Ok(Self {
            name,
            tld,
            full: domain,
        })
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full)
    }
}
