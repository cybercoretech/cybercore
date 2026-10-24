use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const CACHE_TTL: Duration = Duration::from_secs(300);
static CACHE: OnceLock<Mutex<Option<(Instant, Value)>>> = OnceLock::new();

pub async fn snapshot() -> Value {
    if let Some(value) = cached() {
        return value;
    }

    let client = Client::builder()
        .user_agent("Cybercore-Control-Plane/0.8")
        .timeout(Duration::from_secs(8))
        .build();
    let value = match client {
        Ok(client) => json!({
            "cloudflare": cloudflare(&client).await,
            "google": google(&client).await,
            "github": github(&client).await,
        }),
        Err(error) => json!({
            "cloudflare": error_state("ERROR", error.to_string()),
            "google": error_state("ERROR", error.to_string()),
            "github": error_state("ERROR", error.to_string()),
        }),
    };
    if let Ok(mut cache) = CACHE.get_or_init(|| Mutex::new(None)).lock() {
        *cache = Some((Instant::now(), value.clone()));
    }
    value
}

fn cached() -> Option<Value> {
    CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|cache| {
            cache
                .as_ref()
                .filter(|(time, _)| time.elapsed() < CACHE_TTL)
                .map(|(_, value)| value.clone())
        })
}

async fn github(client: &Client) -> Value {
    let Some(token) = std::env::var("GITHUB_TOKEN").ok() else {
        return configured_or_not("GITHUB_TOKEN", "GITHUB_REPOSITORY", "github");
    };
    let Some(repository) = std::env::var_os("GITHUB_REPOSITORY") else {
        return error_state("NOT CONFIGURED", "GITHUB_REPOSITORY is required.");
    };
    let repository = repository.to_string_lossy();
    let response = client
        .get(format!("https://api.github.com/repos/{repository}"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;
    match response {
        Ok(response) if response.status().is_success() => match response.json::<Value>().await {
            Ok(repo) => json!({
                "status": "CONNECTED",
                "source": "GitHub REST API",
                "last_sync": crate::unix_timestamp(),
                "metrics": {
                    "repository": repository,
                    "stars": repo["stargazers_count"],
                    "forks": repo["forks_count"],
                    "open_issues": repo["open_issues_count"],
                    "default_branch": repo["default_branch"],
                    "last_push": repo["pushed_at"]
                }
            }),
            Err(error) => error_state("ERROR", error.to_string()),
        },
        Ok(response) => error_state("ERROR", format!("GitHub returned {}", response.status())),
        Err(error) => error_state("ERROR", error.to_string()),
    }
}

async fn google(client: &Client) -> Value {
    let Some(url) = std::env::var_os("GOOGLE_PAGESPEED_URL") else {
        return configured_or_not(
            "GOOGLE_APPLICATION_CREDENTIALS",
            "GOOGLE_PAGESPEED_URL",
            "google",
        );
    };
    let mut request = client
        .get("https://www.googleapis.com/pagespeedonline/v5/runPagespeed")
        .query(&[
            ("url", url.to_string_lossy().as_ref()),
            ("strategy", "mobile"),
        ]);
    if let Some(key) = std::env::var_os("GOOGLE_PAGESPEED_API_KEY") {
        request = request.query(&[("key", key.to_string_lossy().as_ref())]);
    }
    match request.send().await {
        Ok(response) if response.status().is_success() => match response.json::<Value>().await {
            Ok(report) => {
                let categories = &report["lighthouseResult"]["categories"];
                let audits = &report["lighthouseResult"]["audits"];
                json!({
                    "status": "CONNECTED",
                    "source": "Google PageSpeed Insights API",
                    "last_sync": crate::unix_timestamp(),
                    "metrics": {
                        "url": url.to_string_lossy(),
                        "performance": categories["performance"]["score"],
                        "accessibility": categories["accessibility"]["score"],
                        "best_practices": categories["best-practices"]["score"],
                        "seo": categories["seo"]["score"],
                        "lcp_ms": audits["largest-contentful-paint"]["numericValue"],
                        "cls": audits["cumulative-layout-shift"]["numericValue"],
                        "fcp_ms": audits["first-contentful-paint"]["numericValue"],
                        "ttfb_ms": audits["server-response-time"]["numericValue"]
                    }
                })
            }
            Err(error) => error_state("ERROR", error.to_string()),
        },
        Ok(response) => error_state("ERROR", format!("PageSpeed returned {}", response.status())),
        Err(error) => error_state("ERROR", error.to_string()),
    }
}

async fn cloudflare(client: &Client) -> Value {
    let Some(token) = std::env::var("CLOUDFLARE_API_TOKEN").ok() else {
        return configured_or_not("CLOUDFLARE_API_TOKEN", "CLOUDFLARE_ZONE_ID", "cloudflare");
    };
    let Some(zone_id) = std::env::var_os("CLOUDFLARE_ZONE_ID") else {
        return error_state(
            "NOT CONFIGURED",
            "CLOUDFLARE_ZONE_ID is required for zone metrics.",
        );
    };
    let response = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/settings/ipv4",
            zone_id.to_string_lossy()
        ))
        .bearer_auth(token)
        .header("Accept", "application/json")
        .send()
        .await;
    match response {
        Ok(response) if response.status().is_success() => json!({
            "status": "CONNECTED",
            "source": "Cloudflare REST API",
            "last_sync": crate::unix_timestamp(),
            "metrics": {
                "zone_id_configured": true,
                "api_reachable": true
            }
        }),
        Ok(response)
            if response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::FORBIDDEN =>
        {
            error_state(
                "ERROR",
                "Cloudflare token was rejected or lacks zone permissions.",
            )
        }
        Ok(response) => error_state(
            "ERROR",
            format!("Cloudflare returned {}", response.status()),
        ),
        Err(error) => error_state("ERROR", error.to_string()),
    }
}

fn configured_or_not(secret: &str, target: &str, provider: &str) -> Value {
    if std::env::var_os(secret).is_some() {
        error_state(
            "NOT CONFIGURED",
            format!("{target} is required for {provider} metrics."),
        )
    } else {
        json!({"status": "NOT CONFIGURED", "source": secret, "last_sync": Value::Null})
    }
}

fn error_state(status: &str, message: impl Into<String>) -> Value {
    json!({"status": status, "last_sync": Value::Null, "error": message.into()})
}
