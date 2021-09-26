use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

pub type Id = i64;
pub type Coin = String;
pub type Symbol = String;

#[derive(Clone, Debug, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub result: T,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

// REST API -> Subaccounts

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subaccount {
    pub nickname: String,
    pub deletable: bool,
    pub editable: bool,
    pub competition: bool,
}

pub type Subaccounts = Vec<Subaccount>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Create {
    pub nickname: String,
    pub deletable: bool,
    pub editable: bool,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeName;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin: Coin,
    pub free: Decimal,
    pub total: Decimal,
    pub spot_borrow: Decimal,
    pub available_without_borrow: Decimal,
}

pub type Balances = Vec<Balance>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub id: Id,
    pub coin: Coin,
    pub size: Decimal,
    pub time: DateTime<Utc>,
    pub notes: String,
}

// REST API -> Markets

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MarketType {
    Future,
    Spot,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    #[serde(rename = "type")]
    pub market_type: MarketType,
    pub name: Symbol,
    pub underlying: Option<Coin>,
    pub base_currency: Option<Coin>,
    pub quote_currency: Option<Coin>,
    pub enabled: bool,
    pub ask: Decimal,
    pub bid: Decimal,
    pub last: Decimal,
    pub post_only: bool,
    pub price_increment: Decimal,
    pub size_increment: Decimal,
    pub restricted: bool,
    pub min_provide_size: Decimal,
    pub price: Decimal,
    pub high_leverage_fee_exempt: bool,
    pub change1h: Decimal,
    pub change24h: Decimal,
    pub change_bod: Decimal,
    pub quote_volume24h: Decimal,
    pub volume_usd24h: Decimal,
}

pub type Markets = Vec<Market>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Orderbook {
    pub asks: Vec<(Decimal, Decimal)>,
    pub bids: Vec<(Decimal, Decimal)>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "side", rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: Id,
    pub liquidation: bool,
    pub price: Decimal,
    pub side: Side,
    pub size: Decimal,
    pub time: DateTime<Utc>,
}

pub type Trades = Vec<Trade>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingPayment {
    pub id: Id,
    pub future: String,
    pub payment: Decimal,
    pub time: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub close: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub open: Decimal,
    pub volume: Decimal,
    pub start_time: DateTime<Utc>,
}

pub type Prices = Vec<Price>;

// REST API -> Futures

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FutureType {
    Future,
    Perpetual,
    Prediction,
    Move,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Future {
    pub ask: Option<Decimal>,
    pub bid: Option<Decimal>,
    pub change1h: Option<Decimal>,
    pub change24h: Option<Decimal>,
    pub change_bod: Option<Decimal>,
    pub volume_usd24h: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub description: String,
    pub enabled: bool,
    pub expired: bool,
    pub expiry: Option<DateTime<Utc>>,
    pub index: Option<Decimal>,
    pub imf_factor: Decimal,
    pub last: Option<Decimal>,
    pub lower_bound: Decimal,
    pub mark: Option<Decimal>,
    pub name: Symbol,
    pub perpetual: bool,
    pub position_limit_weight: Decimal,
    pub post_only: bool,
    pub price_increment: Decimal,
    pub size_increment: Decimal,
    pub underlying: Symbol,
    pub upper_bound: Decimal,
    #[serde(rename = "type")]
    pub market_type: FutureType,
}

pub type Futures = Vec<Future>;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureStats {
    pub volume: Decimal,
    pub next_funding_rate: Decimal,
    pub next_funding_time: DateTime<Utc>,
    pub expiration_price: Decimal,
    pub predicted_expiration_price: Decimal,
    pub strike_price: Decimal,
    pub open_interest: Decimal,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingRate {
    pub future: Symbol,
    pub rate: Decimal,
    pub time: DateTime<Utc>,
}

pub type FundingRates = Vec<FundingRate>;

// REST API -> Account

/// Returned by GET /account.
/// See https://docs.ftx.com/#get-account-information.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub backstop_provider: bool,
    pub charge_interest_on_negative_usd: bool,
    pub collateral: Decimal,
    pub free_collateral: Decimal,
    pub initial_margin_requirement: Decimal,
    pub liquidating: bool,
    pub maintenance_margin_requirement: Decimal,
    pub maker_fee: Decimal,
    pub margin_fraction: Option<Decimal>,
    pub open_margin_fraction: Option<Decimal>,
    pub position_limit: Option<Decimal>,
    pub position_limit_used: Option<Decimal>,
    pub taker_fee: Decimal,
    pub total_account_value: Decimal,
    pub total_position_size: Decimal,
    pub use_ftt_collateral: bool,
    pub username: String,
    pub leverage: Decimal,
    pub positions: Vec<Position>,
    pub spot_lending_enabled: bool,
    pub spot_margin_enabled: bool,
}

/// Returned by GET /positions.
/// See https://docs.ftx.com/#get-positions.
pub type Positions = Vec<Position>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub cost: Decimal,
    pub entry_price: Option<Decimal>,
    pub estimated_liquidation_price: Option<Decimal>,
    pub future: String,
    pub initial_margin_requirement: Decimal,
    pub long_order_size: Decimal,
    pub maintenance_margin_requirement: Decimal,
    pub net_size: Decimal,
    pub open_size: Decimal,
    pub realized_pnl: Decimal,
    pub short_order_size: Decimal,
    pub side: Side,
    pub size: Decimal,
    pub unrealized_pnl: Decimal,
    pub collateral_used: Decimal,
}

// REST API -> Wallet
// TODO

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
    pub id: Coin,     // "USDT"
    pub name: String, // "USD Tether"
    pub collateral: bool,
    pub usd_fungible: bool,
    pub is_etf: bool, // Not documented
    pub is_token: bool,
    pub hidden: bool, // Not documented
    pub can_deposit: bool,
    pub can_withdraw: bool,
    pub can_convert: bool,
    pub has_tag: bool,
    pub collateral_weight: Decimal,
    pub fiat: bool,
    pub methods: Vec<String>, // ["omni", "erc20", "trx", "sol", "heco"]
    pub erc20_contract: Option<String>, // "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    pub bep2_asset: Option<String>, // "ETHBEAR-B2B"
    pub trc20_contract: Option<String>, // "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
    pub spl_mint: Option<String>, // "BQcdHdAQW1hczDbBi9hiegXAR7A98Q9jx3X3iBBBDiq4"
    pub credit_to: Option<String>, // "USDT"
    pub spot_margin: bool,    // Not documented
    pub tokenized_equity: Option<bool>, // Not documented
    pub index_price: Decimal, // Not documented; note that ~8% return 1e-8
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDepositAddress {
    pub address: String,
    pub tag: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub coin: String,
    pub free: Decimal,
    pub total: Decimal,
    pub spot_borrow: Decimal,
    pub available_without_borrow: Decimal,
    /// As of 2021-05-12, usdValue is not documented on
    /// https://docs.ftx.com/#get-balances, but it is returned.
    pub usd_value: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub enum DepositStatus {
    Confirmed,
    Unconfirmed,
    Cancelled,
    Complete,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDeposit {
    pub id: Id,
    pub coin: String,
    pub size: Decimal,
    pub time: chrono::DateTime<Utc>,
    pub status: DepositStatus,
    pub confirmations: Option<i64>,
    pub confirmed_time: Option<chrono::DateTime<Utc>>,
    pub fee: Option<Decimal>, // fee, not included in size
    pub txid: Option<String>,
    pub notes: Option<String>,
}

// REST API -> Orders
// TODO

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "order_type", rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "conditional_order_type", rename_all = "snake_case")]
pub enum ConditionalOrderType {
    Stop,
    TrailingStop,
    TakeProfit,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "fill_type", rename_all = "snake_case")]
pub enum FillType {
    Otc,
    Order,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
/// Represents the status of the order.
/// However, the REST and websockets APIs assign these values differently.
///
/// When submitting orders over REST, the API will immediately return whether
/// the order is accepted into FTX's queue for processing, but not the results
/// of the processing. If the order is accepted into the queue, the API will
/// return an `OrderInfo` with `OrderStatus::New`, otherwise it will return an error.
///
/// If the order is rejected during processing (e.g. when submitting a post-only
/// limit order with a price that would be executed as a taker order), the user
/// will not know this unless they do one of the following:
/// - Call the `get_order` REST API to see if the order status has been updated
/// - Listen to orders over websockets to be notified of the update order status
///   as soon as it is available.
/// To get near-immediate feedback on the status of possibly-rejected orders,
/// we recommend subscribing to the `Orders` channel over websockets.
///
/// When listening to orders over websockets, the websockets API will report
/// only the status of the order when it has been processed:
/// - If an order is rejected upon processing, the websockets API will emit an
///   `OrderInfo` with `OrderStatus::Closed`. Unlike the REST API, it will not
///   return an `OrderInfo` with `OrderStatus::New`.
/// - If a limit order is accepted and not immediately filled upon processing,
///   the websockets API will emit an `OrderInfo` with `OrderStatus::New`,
///   confirming the order as active.
/// - If a limit or market order is accepted and filled immediately upon
///   processing, the websockets API emits an `OrderInfo` with
///   `OrderStatus::Closed`.
pub enum OrderStatus {
    /// Rest: accepted but not processed yet; Ws: processed and confirmed active
    New,
    /// Applicable to Rest only
    Open,
    /// Rest: filled or cancelled; Ws: filled, rejected, or cancelled
    Closed,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "conditional_order_status", rename_all = "snake_case")]
pub enum ConditionalOrderStatus {
    Open,
    Cancelled,
    Triggered,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    pub id: Id,
    pub market: String,
    pub future: Option<String>,
    pub r#type: OrderType,
    pub side: Side,
    pub price: Option<Decimal>, // null for new market orders
    pub size: Decimal,
    pub reduce_only: bool,
    pub ioc: bool,
    pub post_only: bool,
    pub status: OrderStatus,
    pub filled_size: Decimal,
    pub remaining_size: Decimal,
    pub avg_fill_price: Option<Decimal>,
    pub liquidation: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub client_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalOrderInfo {
    pub id: Id,
    pub market: String,
    pub future: Option<String>,
    pub order_type: OrderType,
    pub r#type: ConditionalOrderType,
    pub side: Side,
    pub price: Option<Decimal>, // null for new market orders
    pub size: Decimal,
    pub status: ConditionalOrderStatus,
    pub trail_start: Option<Decimal>,
    pub trail_value: Option<Decimal>,
    pub trigger_price: Decimal,
    pub triggered_at: Option<DateTime<Utc>>,
    pub filled_size: Option<Decimal>,
    pub avg_fill_price: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub client_id: Option<String>,
    pub order_price: Option<Decimal>,
    pub retry_until_filled: bool,
}

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FillInfo {
    pub id: Id,
    pub market: Option<String>,
    pub future: Option<String>,
    pub r#type: FillType,
    pub side: Side,
    pub price: Decimal, // null for new market orders
    pub size: Decimal,
    pub time: DateTime<Utc>,
    pub fee: Decimal,
    pub fee_currency: Coin,
    pub fee_rate: Decimal,
    pub liquidity: String,
    pub base_currency: Option<Coin>,
    pub quote_currency: Option<Coin>,
    pub order_id: Option<Id>,
    pub trade_id: Option<Id>,
}
