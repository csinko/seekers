use crate::constants;
use crate::{UsageData, UsageWindow};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    five_hour: Option<ClaudeUsageWindow>,
    seven_day: Option<ClaudeUsageWindow>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageWindow {
    utilization: f64,
    resets_at: Option<String>,
}

pub async fn fetch_usage(org_id: &str, session_key: &str, client: &reqwest::Client) -> Result<UsageData, Box<dyn std::error::Error + Send + Sync>> {

    let url = format!("{}/organizations/{}/usage", constants::CLAUDE_API_BASE, org_id);

    let response = client
        .get(&url)
        .header("Cookie", format!("sessionKey={session_key}"))
        .header("Accept", "application/json")
        .header("User-Agent", constants::USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("API request failed: {status}").into());
    }

    let data: ClaudeUsageResponse = response.json().await?;

    Ok(UsageData {
        five_hour: data.five_hour.map(|w| UsageWindow {
            utilization: w.utilization,
            resets_at: w.resets_at.unwrap_or_default(),
        }),
        seven_day: data.seven_day.map(|w| UsageWindow {
            utilization: w.utilization,
            resets_at: w.resets_at.unwrap_or_default(),
        }),
    })
}


