use super::*;
use dotenv::dotenv;
use rust_decimal_macros::dec;
use std::env::var;

async fn init_api() -> Rest {
    dotenv().ok();

    let subaccount = var("SUBACCOUNT").ok();

    let api = Rest::new(
        var("FTX_API_KEY").expect("API Key is not defined."),
        var("FTX_API_SECRET").expect("API Secret is not defined."),
        subaccount.clone(),
    );

    // Test create subaccount only if credentials are account-wide
    if subaccount.is_none() {
        read_only(api.create_subaccount("Bot").await);
    }

    api
}

fn read_only<T>(result: Result<T>) {
    match result {
        Err(Error::Api(error)) if error == *"Not allowed with read-only permissions" => {}
        _ => panic!("Expected read-only subaccount."),
    }
}

#[tokio::test]
async fn get_subaccounts() {
    let rest = init_api().await;
    if rest.subaccount.is_none() {
        // Test only if credentials are account-wide
        rest.get_subaccounts().await.unwrap();
    }
}

#[tokio::test]
async fn create_subaccount() {
    let rest = init_api().await;
    if rest.subaccount.is_none() {
        // Test only if credentials are account-wide
        read_only(rest.create_subaccount("Bot").await);
    }
}

#[tokio::test]
async fn change_subaccount_name() {
    let rest = init_api().await;
    if rest.subaccount.is_none() {
        // Test only if credentials are account-wide
        read_only(rest.change_subaccount_name("Bot", "Bot").await);
    }
}

#[tokio::test]
async fn delete_subaccount() {
    let rest = init_api().await;
    if rest.subaccount.is_none() {
        // Test only if credentials are account-wide
        read_only(rest.delete_subaccount("Bot").await);
    }
}

#[tokio::test]
async fn get_subaccount_balances() {
    let rest = init_api().await;
    // Test using given subaccount otherwise use "Bot"
    let subaccount = match &rest.subaccount {
        None => "Bot",
        Some(sub) => sub,
    };
    rest.get_subaccount_balances(subaccount).await.unwrap_err();
}

#[tokio::test]
async fn transfer_between_subaccounts() {
    let rest = init_api().await;
    if rest.subaccount.is_none() {
        // Test only if credentials are account-wide
        rest.transfer_between_subaccounts("BTC", Decimal::zero(), "Source", "Destination")
            .await
            .unwrap_err();
    }
}

#[tokio::test]
async fn get_markets() {
    init_api().await.get_markets().await.unwrap();
}

#[tokio::test]
async fn get_market() {
    init_api().await.get_market("BTC/USD").await.unwrap();
}

#[tokio::test]
async fn get_orderbook() {
    init_api()
        .await
        .get_orderbook("BTC/USD", None)
        .await
        .unwrap();
    init_api()
        .await
        .get_orderbook("BTC/USD", Some(50))
        .await
        .unwrap();
}

#[tokio::test]
async fn get_trades() {
    init_api()
        .await
        .get_trades("BTC/USD", None, None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn get_historical_prices() {
    init_api()
        .await
        .get_historical_prices("BTC/USD", 300, None, None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn get_futures() {
    init_api().await.get_futures().await.unwrap();
}

#[tokio::test]
async fn get_future() {
    init_api().await.get_future("BTC-PERP").await.unwrap();
}

#[tokio::test]
async fn account_deserialization() {
    // Sanitized response from FTX API.
    let json = r#"{
        "backstopProvider":false,
        "chargeInterestOnNegativeUsd":false,
        "collateral":123,
        "freeCollateral":0.0,
        "initialMarginRequirement":0.01,
        "leverage":1.0,
        "liquidating":false,
        "maintenanceMarginRequirement":0.03,
        "makerFee":-5e-6,
        "marginFraction":0.01,
        "openMarginFraction":0.01,
        "positionLimit":null,
        "positionLimitUsed":null,
        "positions":[{
            "collateralUsed":123.45,
            "cost":-123.45,
            "entryPrice":1.23,
            "estimatedLiquidationPrice":1.23,
            "future":"FOOBAR",
            "initialMarginRequirement":0.1,
            "longOrderSize":0.0,
            "maintenanceMarginRequirement":0.01,
            "netSize":12345,
            "openSize":12345,
            "realizedPnl":12345,
            "shortOrderSize":0.0,
            "side":"sell",
            "size":12345,
            "unrealizedPnl":0.0
        }],
        "spotLendingEnabled":true,
        "spotMarginEnabled":true,
        "takerFee":0.0001,
        "totalAccountValue":123.1,
        "totalPositionSize":123.1,
        "useFttCollateral":true,
        "username":"foo@example.com"
    }"#;
    let _account: Account = serde_json::from_str(json).unwrap();
}

#[tokio::test]
async fn get_coins() {
    init_api().await.get_coins().await.unwrap();
}

#[tokio::test]
async fn place_modify_cancel_orders() {
    manipulate_orders().await;
}

// Helper function used in place_modify_cancel_orders and ws::tests::orders
pub async fn manipulate_orders() {
    let api = init_api().await;
    // Testing with ETH since BTC's minimum provide (maker) size is 0.01 BTC,
    // too large for testing purposes
    let market = String::from("ETH-PERP");
    let price = api.get_market(market.as_str()).await.unwrap().price;

    // Bid size will start at 0.001, which is ETH-PERP's minimum size increment
    let initial_bid_size = dec!(0.001);
    // Bid size will double in the modified order
    let modified_bid_size = dec!(0.002);

    // Bid at 95% of the current price
    let initial_bid_price = dec!(0.95) * price;
    // Bid will be modified to 94% of the current price
    let modified_bid_price = dec!(0.94) * price;

    // Round to 0.1, which is ETH-PERP's minimum price increment
    let initial_bid_price = initial_bid_price.round_dp(1);
    let modified_bid_price = modified_bid_price.round_dp(1);
    // println!("Bid price: {}", initial_bid_price);
    // println!("Modified bid price: {}", modified_bid_price);

    // Test place order
    let initial_order = api
        .place_order(
            market.as_str(),
            Side::Buy,
            Some(initial_bid_price),
            OrderType::Limit,
            initial_bid_size,
            None,
            None,
            Some(true),
            None,
        )
        .await
        .unwrap();
    // println!("Initial order: {:?}", initial_order);
    assert_eq!(initial_bid_price, initial_order.price.unwrap());
    assert_eq!(initial_bid_size, initial_order.size);

    // Test modify order
    let modified_order = api
        .modify_order(
            initial_order.id,
            Some(modified_bid_price),
            Some(modified_bid_size),
            None,
        )
        .await
        .unwrap();
    // println!("Modified order: {:?}", modified_order);
    // Order ID is different for the modified order because FTX implements modify
    // as cancelling the order and placing a new one
    assert_ne!(initial_order.id, modified_order.id);
    assert_eq!(modified_bid_price, modified_order.price.unwrap());
    assert_eq!(modified_bid_size, modified_order.size);

    // Test cancel order
    let cancelled_response = api.cancel_order(modified_order.id).await.unwrap();
    // println!("Cancelled response: {:?}", cancelled_response);
    assert_eq!(
        "Order queued for cancellation".to_string(),
        cancelled_response
    );

    // Check that order was actually cancelled
    let cancelled_order = api.get_order(modified_order.id).await.unwrap();
    // println!("Cancelled order: {:?}", cancelled_order);
    assert_eq!(modified_order.id, cancelled_order.id);
    assert_eq!(dec!(0), cancelled_order.filled_size);
    assert_eq!(None, cancelled_order.avg_fill_price);
    assert_eq!(OrderStatus::Closed, cancelled_order.status);

    // Place a post-only order that will be rejected
    let rejected_bid_price = dec!(1.1) * price; // Bid at 110% of current price
    let rejected_order = api
        .place_order(
            market.as_str(),
            Side::Buy,
            Some(rejected_bid_price),
            OrderType::Limit,
            initial_bid_size,
            None,
            None,
            Some(true),
            None,
        )
        .await
        .unwrap();
    assert_eq!(dec!(0), rejected_order.filled_size);
    assert_eq!(None, rejected_order.avg_fill_price);

    assert_eq!(OrderStatus::New, rejected_order.status);
}
