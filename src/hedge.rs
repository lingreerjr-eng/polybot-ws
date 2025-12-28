use anyhow::Result;
use rs_clob_client::{
    client::ClobClient,
    types::{OrderArgs, OrderSide, OrderType},
};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

use crate::book::TopOfBook;

fn is_filled(resp: &serde_json::Value) -> bool {
    resp.get("status") == Some(&"matched".into())
        || resp.get("orderID").is_some()
}

pub async fn handle_mismatch(
    client: &ClobClient,
    sym: &str,
    token_a: &str,
    token_b: &str,
    px_a: f64,
    px_b: f64,
    shares: f64,
    resp: &Vec<serde_json::Value>,
    books: &HashMap<String, TopOfBook>,
    hedge_sum_max: f64,
) -> Result<()> {
    if resp.len() < 2 {
        return Ok(());
    }

    let a_ok = is_filled(&resp[0]);
    let b_ok = is_filled(&resp[1]);

    if a_ok == b_ok {
        return Ok(());
    }

    let (filled_token, missing_token, filled_px) = if a_ok {
        (token_a, token_b, px_a)
    } else {
        (token_b, token_a, px_b)
    };

    log::warn!(
        "mismatch | {} filled={} missing={} shares={}",
        sym, filled_token, missing_token, shares
    );

    // ---- Refill ----
    if let Some(tob) = books.get(missing_token) {
        if let Some((ask_px, _)) = tob.ask {
            if ask_px <= hedge_sum_max - filled_px {
                let order = client.build_order(OrderArgs {
                    token_id: missing_token.into(),
                    side: OrderSide::Buy,
                    price: ask_px,
                    size: shares,
                });

                let r = client
                    .post_orders(vec![order], OrderType::FillOrKill)
                    .await;

                if r.is_ok() {
                    log::warn!("refill success {}", missing_token);
                    return Ok(());
                }
            }
        }
    }

    sleep(Duration::from_millis(150)).await;

    // ---- Unwind ----
    if let Some(tob) = books.get(filled_token) {
        if let Some((bid_px, _)) = tob.bid {
            let order = client.build_order(OrderArgs {
                token_id: filled_token.into(),
                side: OrderSide::Sell,
                price: bid_px.max(0.001),
                size: shares,
            });

            let _ = client
                .post_orders(vec![order], OrderType::ImmediateOrCancel)
                .await;

            log::warn!("unwind submitted {}", filled_token);
        }
    }

    Ok(())
}

