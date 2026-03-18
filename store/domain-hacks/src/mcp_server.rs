use async_trait::async_trait;
use lens::{LensContext, LensError, McpServerLens, ToolCallRequest, ToolCallResponse};
use crate::domains::{check_domain_availability, check_domains_batch, DomainChecker};
use crate::error::Result;
use crate::strategies::StrategyGenerator;
use crate::types::{Domain, DomainAvailability, StrategyRecommendations};
use serde_json::{json, Value};

pub struct DomainMcpServer {
    godaddy_api_key: Option<String>,
    godaddy_api_secret: Option<String>,
}

impl DomainMcpServer {
    pub fn new() -> Self {
        Self {
            godaddy_api_key: std::env::var("GODADDY_API_KEY").ok(),
            godaddy_api_secret: std::env::var("GODADDY_API_SECRET").ok(),
        }
    }

    pub fn set_credentials(&mut self, key: String, secret: String) {
        self.godaddy_api_key = Some(key);
        self.godaddy_api_secret = Some(secret);
    }

    fn get_credentials(&self) -> Result<(&str, &str)> {
        let key = self.godaddy_api_key.as_ref().ok_or_else(|| {
            crate::error::DomainError::Config(
                "GODADDY_API_KEY not set".to_string()
            )
        })?;
        let secret = self.godaddy_api_secret.as_ref().ok_or_else(|| {
            crate::error::DomainError::Config(
                "GODADDY_API_SECRET not set".to_string()
            )
        })?;
        Ok((key.as_str(), secret.as_str()))
    }

    async fn handle_check_availability(&self, domain: &str) -> Result<Value> {
        let (key, secret) = self.get_credentials()?;
        let result = check_domain_availability(domain, key, secret).await?;

        Ok(json!({
            "domain": result.domain.full,
            "available": result.available,
            "verified": result.verified,
            "price_estimate": result.price_estimate,
            "dns_resolvable": result.dns_status.resolvable,
            "http_accessible": result.http_status.accessible,
            "registration_url": result.registration_url,
        }))
    }

    async fn handle_check_batch(&self, domains: &[String]) -> Result<Value> {
        let (key, secret) = self.get_credentials()?;
        let results = check_domains_batch(domains, key, secret).await?;

        let available: Vec<_> = results.iter().filter(|r| r.available && r.verified).collect();
        let unavailable: Vec<_> = results.iter().filter(|r| !r.available).collect();

        Ok(json!({
            "total": results.len(),
            "available": available.len(),
            "unavailable": unavailable.len(),
            "results": results,
        }))
    }

    async fn handle_generate_strategy(&self, domain: &str) -> Result<Value> {
        let parsed = Domain::parse(domain)?;
        let generator = StrategyGenerator;
        let phase_output = generator.generate_all(&parsed)?;

        let strategies = match phase_output {
            crate::types::PhaseOutput::StrategyGeneration { strategies } => strategies,
            _ => return Err(crate::error::DomainError::StrategyGenerationFailed(
                "Failed to generate strategies".to_string()
            )),
        };

        Ok(json!({
            "domain": parsed.full,
            "tier_1_platforms": strategies.tier_1_platforms.len(),
            "tier_2_actions": strategies.tier_2_actions.len(),
            "tier_3_categories": strategies.tier_3_categories.len(),
            "total_cost_estimate": strategies.total_cost_estimate,
            "strategies": strategies,
        }))
    }
}

impl Default for DomainMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpServerLens for DomainMcpServer {
    fn tool_list(&self) -> Vec<String> {
        vec![
            "check_domain_availability".to_string(),
            "check_domains_batch".to_string(),
            "generate_strategy".to_string(),
            "suggest_domains".to_string(),
        ]
    }

    fn call_tool(&self, request: ToolCallRequest) -> std::result::Result<Value, LensError> {
        let tool_name = request.tool_name.as_str();
        let params = request.params;

        let result = match tool_name {
            "check_domain_availability" => {
                let domain = params.get("domain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LensError::InvalidParams(
                        "domain parameter required".to_string()
                    ))?;

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(self.handle_check_availability(domain))
                })
            }
            "check_domains_batch" => {
                let domains = params.get("domains")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| LensError::InvalidParams(
                        "domains parameter required (array)".to_string()
                    ))?;

                let domain_strings: std::result::Result<Vec<String>, _> = domains
                    .iter()
                    .map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let domains_vec = domain_strings.map_err(|_| LensError::InvalidParams(
                    "domains must be array of strings".to_string()
                ))?;

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(self.handle_check_batch(&domains_vec))
                })
            }
            "generate_strategy" => {
                let domain = params.get("domain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LensError::InvalidParams(
                        "domain parameter required".to_string()
                    ))?;

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(self.handle_generate_strategy(domain))
                })
            }
            "suggest_domains" => {
                let query = params.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LensError::InvalidParams(
                        "query parameter required".to_string()
                    ))?;

                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);

                let (key, secret) = self.get_credentials()?;
                let checker = DomainChecker::new(key.to_string(), secret.to_string());
                let suggestions = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(checker.suggest_domains(query, limit.unwrap_or(10)))
                });

                Ok(json!({
                    "query": query,
                    "suggestions": suggestions.unwrap_or_default(),
                    "count": suggestions.unwrap_or_default().len(),
                }))
            }
            _ => Err(LensError::ToolNotFound(tool_name.to_string())),
        };

        result.map_err(|e| LensError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_list() {
        let server = DomainMcpServer::new();
        let tools = server.tool_list();

        assert_eq!(tools.len(), 4);
        assert!(tools.contains(&"check_domain_availability".to_string()));
        assert!(tools.contains(&"check_domains_batch".to_string()));
        assert!(tools.contains(&"generate_strategy".to_string()));
        assert!(tools.contains(&"suggest_domains".to_string()));
    }

    #[test]
    fn test_credentials_validation() {
        let mut server = DomainMcpServer::new();
        assert!(server.get_credentials().is_err());

        server.set_credentials("test_key".to_string(), "test_secret".to_string());
        let (key, secret) = server.get_credentials().unwrap();
        assert_eq!(key, "test_key");
        assert_eq!(secret, "test_secret");
    }
}
