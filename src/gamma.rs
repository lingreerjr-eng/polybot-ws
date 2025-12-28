use anyhow::{Result, bail};
use serde::Deserialize;
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MarketPair {
    pub symbol: String,
    pub slug: String,
    pub condition_id: String,
    pub token_a: String,
    pub token_b: String,
}

#[derive(Deserialize)]
struct GammaEvent {
    markets: Vec<GammaMarket>,
}

#[derive(Deserialize)]
struct GammaMarket {
    #[serde(rename = "conditionId")]
    condition_id: String,
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: serde_json::Value,
    #[serde(default)]
    acceptingOrders: bool,
}

fn now_15m_epoch() -> i64 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    (t / 900) * 900
}

fn parse_token_ids(v: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = v.as_array() {
        return arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
    }
    if let Some(s) = v.as_str() {
        if s.starts_with('[') {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(s) {
                return arr;
            }
        }
        return s.split(',')
            .map(|x| x.trim().trim_matches('"').to_string())
            .filter(|x| !x.is_empty())
            .collect();
    }
    vec![]
}

pub async fn resolve_pairs(
    gamma_host: &str,
) -> Result<Vec<MarketPair>> {
    let client = Client::new();
    let base = now_15m_epoch();

    let prefixes = vec![
        ("BTC", "btc-updown-15m-"),
        ("ETH", "eth-updown-15m-"),
        ("SOL", "sol-updown-15m-"),
        ("XRP", "xrp-updown-15m-"),
    ];

    let mut out = Vec::new();

    for (sym, prefix) in prefixes {
        let mut resolved = None;

        for offset in [0, 900, 1800] {
            let slug = format!("{}{}", prefix, base + offset);
            let url = format!("{}/events/slug/{}", gamma_host, slug);

            let resp = client.get(&url).send().await?;
            if !resp.status().is_success() {
                continue;
            }

            let ev: GammaEvent = resp.json().await?;
            let mkt = ev.markets.first().ok_or_else(|| bail!("no markets"))?;

            if !mkt.acceptingOrders {
                continue;
            }

            let tokens = parse_token_ids(&mkt.clob_token_ids);
            if tokens.len() != 2 {
                continue;
            }

            resolved = Some(MarketPair {
                symbol: sym.to_string(),
                slug,
                condition_id: mkt.condition_id.clone(),
                token_a: tokens[0].clone(),
                token_b: tokens[1].clone(),
            });
            break;
        }

        if let Some(p) = resolved {
            out.push(p);
        }
    }

    Ok(out)
}

