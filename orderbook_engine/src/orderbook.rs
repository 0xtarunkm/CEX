use std::collections::{BTreeMap, VecDeque};

use crate::types::{Order, OrderbookResult, Trade};

pub enum OrderbookMessage {
    PlaceOrder(Order),
    CancelOrder(u64),
    GetDepth,
}

pub struct Orderbook {
    market: String,
    buy_orders: BTreeMap<u64, VecDeque<Order>>,
    sell_orders: BTreeMap<u64, VecDeque<Order>>,
    last_trade_id: u64,
}

impl Orderbook {
    pub fn new(market: String) -> Self {
        Orderbook {
            market,
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
            last_trade_id: 0,
        }
    }

    // pub fn process(&mut self, message: OrderbookMessage) -> OrderbookResult {
    //     match message {
    //         OrderbookMessage::PlaceOrder(order) => OrderbookResult::Trades(trades),
    //         OrderbookMessage::CancelOrder(order_id) => {
    //             // if let Some(cancelled_order) = self.cancel_order(order_id) {
    //             //     OrderbookResult::CancelledOrder(cancelled_order)
    //             // } else {
    //             OrderbookResult::OrderNotFound
    //             // }
    //         }
    //         OrderbookMessage::GetDepth => {
    //             OrderbookResult::Depth(())
    //         }
    //     }
    // }

    fn match_buy_order(&mut self, trades: &mut Vec<Trade>, mut order: Order) {
        while let Some((&price, sell_orders)) = self.sell_orders.iter_mut().next() {
            if price > order.price || order.quantity == order.filled {
                break;
            }

            while let Some(sell_order) = sell_orders.front_mut() {
                let match_quantity = std::cmp::min(
                    order.quantity - order.filled,
                    sell_order.quantity - sell_order.filled,
                );

                if match_quantity > 0 {
                    order.filled += match_quantity;
                    sell_order.filled += match_quantity;

                    trades.push(Trade {
                        id: self.last_trade_id,
                        price,
                        quantity: match_quantity,
                        taker_order_id: order.id,
                        maker_order_id: sell_order.id,
                        maker_user_id: sell_order.user_id.clone(),
                        taker_user_id: order.user_id.clone(),
                    });
                    self.last_trade_id += 1;
                }

                if sell_order.filled == sell_order.quantity {
                    sell_orders.pop_front();
                } else {
                    break;
                }

                if order.filled == order.quantity {
                    break;
                }
            }

            if sell_orders.is_empty() {
                self.sell_orders.remove(&price);
            }
        }

        if order.filled < order.quantity {
            self.buy_orders
                .entry(order.price)
                .or_default()
                .push_back(order);
        }
    }
}
