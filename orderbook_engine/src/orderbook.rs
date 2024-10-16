use std::collections::{BTreeMap, VecDeque};

use crate::types::{Order, Side, Trade};

pub enum OrderbookMessage {
    PlaceOrder(Order),
    CancelOrder(u64),
}

pub struct Orderbook {
    market: String,
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
    last_trade_id: u64,
}

impl Orderbook {
    pub fn new(market: String) -> Self {
        Orderbook {
            market,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_trade_id: 0,
        }
    }

    pub async fn process(&mut self, message: OrderbookMessage) {
        match message {
            OrderbookMessage::PlaceOrder(order) => {
                let trades = self.place_order(order);
                println!("trades: {:?}", trades);
            }
            OrderbookMessage::CancelOrder(order_id) => {
                if let Some(cancelled_order) = self.cancel_order(order_id) {
                    println!("Cancelled order: {:?}", cancelled_order);
                }
            }
        }
    }

    pub fn place_order(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => self.match_bids(&mut trades, order),
            Side::Sell => self.match_asks(&mut trades, order),
        }

        trades
    }

    fn match_bids(&mut self, trades: &mut Vec<Trade>, mut order: Order) {
        while let Some((&price, asks)) = self.asks.iter_mut().next() {
            if price > order.price || order.quantity == order.filled {
                break;
            }

            while let Some(sell_order) = asks.front_mut() {
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
                    asks.pop_front();
                } else {
                    break;
                }

                if order.filled == order.quantity {
                    break;
                }
            }

            if asks.is_empty() {
                self.asks.remove(&price);
            }
        }

        if order.filled < order.quantity {
            self.bids
                .entry(order.price)
                .or_default()
                .push_back(order);
        }
    }

    fn match_asks(&mut self, trades: &mut Vec<Trade>, mut order: Order) {
        while let Some((&price, bids)) = self.bids.iter_mut().next_back() {
            if price < order.price || order.quantity == order.filled {
                break;
            }

            while let Some(buy_order) = bids.front_mut() {
                let match_quantity = std::cmp::min(
                    order.quantity - order.filled,
                    buy_order.quantity - buy_order.filled,
                );

                if match_quantity > 0 {
                    order.filled += match_quantity;
                    buy_order.filled += match_quantity;

                    trades.push(Trade {
                        id: self.last_trade_id,
                        price,
                        quantity: match_quantity,
                        maker_order_id: buy_order.id,
                        taker_order_id: order.id,
                        maker_user_id: buy_order.user_id.clone(),
                        taker_user_id: order.user_id.clone(),
                    });
                    self.last_trade_id += 1;
                }

                if buy_order.filled == buy_order.quantity {
                    bids.pop_front();
                } else {
                    break;
                }

                if order.filled == order.quantity {
                    break;
                }
            }

            if bids.is_empty() {
                self.bids.remove(&price);
            }
        }

        if order.filled < order.quantity {
            self.asks
                .entry(order.price)
                .or_default()
                .push_back(order);
        }
    }

    fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        let mut cancelled_order = None;

        for orders in self
            .bids
            .values_mut()
            .chain(self.asks.values_mut())
        {
            if let Some(pos) = orders.iter().position(|order| order.id == order_id) {
                cancelled_order = Some(orders.remove(pos).unwrap());
                break;
            }
        }

        cancelled_order
    }

    pub fn get_depth(&self) -> (Vec<(u64, u64)>, Vec<(u64, u64)>) {
        let buy_depth: Vec<(u64, u64)> = self
            .bids
            .iter()
            .map(|(&price, orders)| (price, orders.iter().map(|o| o.quantity - o.filled).sum()))
            .collect();

        let sell_depth: Vec<(u64, u64)> = self
            .asks
            .iter()
            .map(|(&price, orders)| (price, orders.iter().map(|o| o.quantity - o.filled).sum()))
            .collect();

        (buy_depth, sell_depth)
    }
}
