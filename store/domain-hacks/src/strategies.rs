use crate::error::Result;
use crate::types::{Domain, DomainStrategy, PhaseOutput, StrategyRecommendations, StrategyTier};

pub struct StrategyGenerator;

impl StrategyGenerator {
    pub fn generate_all(&self, domain: &Domain) -> Result<PhaseOutput> {
        let tier_1 = self.generate_tier_1(domain)?;
        let tier_2 = self.generate_tier_2(domain)?;
        let tier_3 = self.generate_tier_3(domain)?;

        let total_cost =
            tier_1.len() as f64 * 12.5 + tier_2.len() as f64 * 12.5 + tier_3.len() as f64 * 12.5;

        Ok(PhaseOutput::StrategyGeneration {
            strategies: StrategyRecommendations {
                tier_1_platforms: tier_1,
                tier_2_actions: tier_2,
                tier_3_categories: tier_3,
                total_cost_estimate: total_cost,
            },
        })
    }

    pub fn generate_tier_1(&self, domain: &Domain) -> Result<Vec<DomainStrategy>> {
        let platform_strategies = match domain.name.as_str() {
            "yt" | "youtube" | "videoai" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier1Platform,
                    rationale: "YouTube is the #2 search engine. 'yt.*' domains capture developer tool search intent directly from URL bar and autocomplete.".to_string(),
                    target_audience: "YouTube creators, video editors, content teams".to_string(),
                    use_cases: vec![
                        "Video transcription & subtitle tools".to_string(),
                        "Thumbnail generators".to_string(),
                        "Analytics dashboards".to_string(),
                        "Comment management systems".to_string(),
                        "Channel growth automation".to_string(),
                    ],
                    seo_keywords: vec![
                        "YouTube developer tools".to_string(),
                        "YouTube automation".to_string(),
                        "video transcription API".to_string(),
                        "YouTube analytics tools".to_string(),
                    ],
                    traffic_potential: 2_400_000_000.0,
                    monetization_paths: vec![
                        "SaaS subscriptions ($19-49/mo)".to_string(),
                        "API usage billing".to_string(),
                        "Enterprise teams".to_string(),
                        "Creator economy marketplace".to_string(),
                    ],
                },
            ],
            "x" | "twitter" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier1Platform,
                    rationale: "X.com is transitioning to 'everything app'. 'x.*' domains capture dev tools for this emerging ecosystem.".to_string(),
                    target_audience: "X developers, content teams, businesses".to_string(),
                    use_cases: vec![
                        "Thread scheduling & automation".to_string(),
                        "Analytics & sentiment analysis".to_string(),
                        "Bot development tools".to_string(),
                        "Community management".to_string(),
                        "X API integrations".to_string(),
                    ],
                    seo_keywords: vec![
                        "X developer tools".to_string(),
                        "Twitter automation tools".to_string(),
                        "X API integration".to_string(),
                    ],
                    traffic_potential: 450_000_000.0,
                    monetization_paths: vec![
                        "SaaS platform ($29-99/mo)".to_string(),
                        "API services".to_string(),
                        "Enterprise social stack".to_string(),
                    ],
                },
            ],
            "ig" | "instagram" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier1Platform,
                    rationale: "Instagram is moving toward creator tools. 'ig.*' domains capture planning/scheduling/search intent.".to_string(),
                    target_audience: "Creators, brands, agencies".to_string(),
                    use_cases: vec![
                        "Content scheduling".to_string(),
                        "Story planning tools".to_string(),
                        "Analytics & insights".to_string(),
                        "UGC management".to_string(),
                        "Influencer collaboration".to_string(),
                    ],
                    seo_keywords: vec![
                        "Instagram scheduler".to_string(),
                        "Creator tools Instagram".to_string(),
                        "IG analytics platform".to_string(),
                    ],
                    traffic_potential: 1_800_000_000.0,
                    monetization_paths: vec![
                        "Creator platform ($15-49/mo)".to_string(),
                        "Agency dashboard".to_string(),
                        "Brand toolkit".to_string(),
                    ],
                },
            ],
            "devboosthub" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier1Platform,
                    rationale: "Centralize dev tools across categories. Single hub for developer productivity reduces switching cost and increases retention.".to_string(),
                    target_audience: "Developers, teams, dev shops".to_string(),
                    use_cases: vec![
                        "Tool directory & discovery".to_string(),
                        "Productivity dashboards".to_string(),
                        "Workflow automation".to_string(),
                        "Team collaboration".to_string(),
                        "Learning resources".to_string(),
                    ],
                    seo_keywords: vec![
                        "developer tools hub".to_string(),
                        "dev productivity platform".to_string(),
                        "developer tool directory".to_string(),
                        "boost developer workflow".to_string(),
                    ],
                    traffic_potential: 12_000_000.0,
                    monetization_paths: vec![
                        "Freemium SaaS (free + $20/mo)".to_string(),
                        "Team plans ($49-199/mo)".to_string(),
                        "Tool listings ($99-299/mo)".to_string(),
                        "Enterprise workspace".to_string(),
                    ],
                },
            ],
            _ => vec![],
        };

        Ok(platform_strategies)
    }

    pub fn generate_tier_2(&self, domain: &Domain) -> Result<Vec<DomainStrategy>> {
        let action_strategies = match domain.name.as_str() {
            "buildit" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier2Action,
                    rationale: "Instant action verb 'build it' captures developer mindset. 'buildit.dev' is memorable and action-oriented.".to_string(),
                    target_audience: "Full-stack developers, indie hackers, makers".to_string(),
                    use_cases: vec![
                        "AI-assisted code generation".to_string(),
                        "Deployment pipelines".to_string(),
                        "Project templates".to_string(),
                        "Collaborative workspaces".to_string(),
                    ],
                    seo_keywords: vec![
                        "build it faster".to_string(),
                        "developer productivity".to_string(),
                        "ship code faster".to_string(),
                        "instant deployment".to_string(),
                    ],
                    traffic_potential: 8_500_000.0,
                    monetization_paths: vec![
                        "Pro developer tools ($29-79/mo)".to_string(),
                        "Team workspaces ($99-299/mo)".to_string(),
                        "API platform".to_string(),
                    ],
                },
            ],
            "shipit" | "maketoday" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier2Action,
                    rationale: format!("'{}' captures maker psychology - daily inspiration and achievement. High recall for developer community.", domain.name),
                    target_audience: "Makers, indie hackers, creators".to_string(),
                    use_cases: vec![
                        "Daily shipping tracker".to_string(),
                        "Maker community".to_string(),
                        "Build-in-public tools".to_string(),
                        "Achievement badges".to_string(),
                    ],
                    seo_keywords: vec![
                        "ship your project today".to_string(),
                        "maker tools".to_string(),
                        "indie hacker resources".to_string(),
                        "build in public".to_string(),
                    ],
                    traffic_potential: 6_500_000.0,
                    monetization_paths: vec![
                        "Community platform ($9-29/mo)".to_string(),
                        "Premium analytics ($19-49/mo)".to_string(),
                        "Creator marketplace (30% fee)".to_string(),
                    ],
                },
            ],
            "codeit" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier2Action,
                    rationale: "'code it' is developer call-to-action. 'codeit.io' is premium TLD + memorable action phrase.".to_string(),
                    target_audience: "Developers, coding bootcamps, learners".to_string(),
                    use_cases: vec![
                        "Code generation & completion".to_string(),
                        "Learning platform".to_string(),
                        "Code review automation".to_string(),
                        "Pair programming tools".to_string(),
                    ],
                    seo_keywords: vec![
                        "code it online".to_string(),
                        "AI code assistant".to_string(),
                        "developer learning".to_string(),
                        "automated coding".to_string(),
                    ],
                    traffic_potential: 15_000_000.0,
                    monetization_paths: vec![
                        "Freemium coding tools".to_string(),
                        "Learning subscriptions ($19-49/mo)".to_string(),
                        "Enterprise code review".to_string(),
                    ],
                },
            ],
            _ => vec![],
        };

        Ok(action_strategies)
    }

    pub fn generate_tier_3(&self, domain: &Domain) -> Result<Vec<DomainStrategy>> {
        let category_strategies = match domain.name.as_str() {
            "figma-tools" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier3Category,
                    rationale: "Figma ecosystem is exploding. Category authority domain 'figma-tools.dev' positions as the directory.".to_string(),
                    target_audience: "Designers, product teams, Figma users".to_string(),
                    use_cases: vec![
                        "Plugin directory".to_string(),
                        "Component library".to_string(),
                        "Design token management".to_string(),
                        "Export tools".to_string(),
                    ],
                    seo_keywords: vec![
                        "Figma plugins directory".to_string(),
                        "Figma tools list".to_string(),
                        "design tools Figma".to_string(),
                        "Figma plugin marketplace".to_string(),
                    ],
                    traffic_potential: 9_000_000.0,
                    monetization_paths: vec![
                        "Featured listings ($199-499/mo)".to_string(),
                        "Premium directory access ($9-19/mo)".to_string(),
                        "Design service marketplace (30% fee)".to_string(),
                    ],
                },
            ],
            "terminal-tools" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier3Category,
                    rationale: "Terminal is core to developer workflow. 'terminal-tools.dev' positions as the hub for CLI tools.".to_string(),
                    target_audience: "DevOps engineers, backend developers, sysadmins".to_string(),
                    use_cases: vec![
                        "Tool directory".to_string(),
                        "Command validation".to_string(),
                        "Shell script marketplace".to_string(),
                        "Terminal UI library".to_string(),
                    ],
                    seo_keywords: vec![
                        "terminal tools directory".to_string(),
                        "CLI tools list".to_string(),
                        "shell command tools".to_string(),
                        "developer terminal".to_string(),
                    ],
                    traffic_potential: 7_500_000.0,
                    monetization_paths: vec![
                        "Tool marketplace (20% fee)".to_string(),
                        "Pro tools suite ($19-49/mo)".to_string(),
                        "Enterprise terminal stack".to_string(),
                    ],
                },
            ],
            "graphic-tools" => vec![
                DomainStrategy {
                    domain: domain.full.clone(),
                    tier: StrategyTier::Tier3Category,
                    rationale: "Graphic AI tools are trending. Category authority for AI-assisted visual asset creation.".to_string(),
                    target_audience: "Designers, marketers, content creators".to_string(),
                    use_cases: vec![
                        "AI image generator".to_string(),
                        "Template library".to_string(),
                        "Prompt marketplace".to_string(),
                        "Brand kit builder".to_string(),
                    ],
                    seo_keywords: vec![
                        "AI graphic tools".to_string(),
                        "image generation AI".to_string(),
                        "graphic design automation".to_string(),
                        "AI design tools".to_string(),
                    ],
                    traffic_potential: 11_000_000.0,
                    monetization_paths: vec![
                        "AI generation credits ($5-49)".to_string(),
                        "Subscription plans ($19-79/mo)".to_string(),
                        "Template marketplace (30% fee)".to_string(),
                    ],
                },
            ],
            _ => vec![],
        };

        Ok(category_strategies)
    }
}

impl Default for StrategyGenerator {
    fn default() -> Self {
        Self
    }
}
