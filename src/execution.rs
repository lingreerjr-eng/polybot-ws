use rs_clob_client::{
    client::ClobClient,
    types::{OrderArgs, OrderSide, OrderType, SignedOrder},
};
use anyhow::Result;

pub async fn post_fok_pair_presigned(
    client: &ClobClient,
    token_a: &str,
    token_b: &str,
    price_a: f64,
    price_b: f64,
    size: f64,
    dry_run: bool,
) -> Result<Vec<serde_json::Value>> {
    if dry_run {
        log::warn!(
            "DRY RUN | BUY {} @ {} | BUY {} @ {} | size={}",
            token_a, price_a, token_b, price_b, size
        );
        return Ok(vec![]);
    }

    // Build unsigned orders
    let oa = OrderArgs {
        token_id: token_a.into(),
        side: OrderSide::Buy,
        price: price_a,
        size,
    };
    let ob = OrderArgs {
        token_id: token_b.into(),
        side: OrderSide::Buy,
        price: price_b,
        size,
    };

    // 🔑 Pre-sign locally (removes latency)
    let sa: SignedOrder = client.sign_order(oa)?;
    let sb: SignedOrder = client.sign_order(ob)?;

    // Submit atomically
    let resp = client
        .post_signed_orders(vec![sa, sb], OrderType::FillOrKill)
        .await?;

    Ok(resp)
}

