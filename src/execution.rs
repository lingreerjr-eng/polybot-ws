
use rs_clob_client::{
    client::ClobClient,
    types::{OrderArgs, OrderSide, OrderType},
};
use anyhow::Result;

pub async fn post_fok_pair(
    client: &ClobClient,
    token_a: &str,
    token_b: &str,
    price_a: f64,
    price_b: f64,
    size: f64,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        log::warn!("DRY RUN: {} {} @ {}, {} @ {}", size, token_a, price_a, token_b, price_b);
        return Ok(());
    }

    let orders = vec![
        client.build_order(OrderArgs {
            token_id: token_a.into(),
            side: OrderSide::Buy,
            price: price_a,
            size,
        }),
        client.build_order(OrderArgs {
            token_id: token_b.into(),
            side: OrderSide::Buy,
            price: price_b,
            size,
        }),
    ];

    client
        .post_orders(orders, OrderType::FillOrKill)
        .await?;

    Ok(())
}
