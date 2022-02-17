use std::f32;

use crate::utils::map_range;
use crate::candlestick::Candlestick;
use crate::backtest::trade::TradeDirection;
use crate::backtest::trade_rule::TradeRule;

pub trait TradingStrategy {
    /// evaluates if a new Long or Short trade should be executed based on the last candlesticks provided
    /// followed by take profit price and stoploss price
    fn should_start_trade(&mut self) -> Option<(TradeDirection, f32, f32)>;

    // interpret a new price datapoint
    fn new_candlestick(&mut self, candle: &Candlestick);

    //reset all interval parameters to run the strategy again with a new candlesticks
    fn reset(&mut self);

    // returns the current trade direction
    fn percentage_amount_per_trade(&self) -> f32;

    // amount of leverage used in trades 
    fn leverage(&self) -> u8;
}

pub struct SingleStrategy {
    leverage: u8,
    long_rule: TradeRule,
    short_rule: TradeRule,
    percentage_amount_per_trade: f32,
    start_long_trade: bool,
    start_short_trade: bool,
}


impl TradingStrategy for SingleStrategy {
    // checks if a trade should be open and returns the direction with a takeprofit and a stoploss price targets
    fn should_start_trade(&mut self) -> Option<(TradeDirection, f32, f32)> {
        if self.start_long_trade && self.start_short_trade {
            return Option::None;
        }else if self.start_long_trade {
            return Option::Some((TradeDirection::Long, 
                self.long_rule.evaluate_take_profit(), 
                self.long_rule.evaluate_stop_loss()));
        }else if self.start_short_trade {
            return Option::Some((TradeDirection::Short, 
                self.short_rule.evaluate_take_profit(), 
                self.short_rule.evaluate_stop_loss()));
        }
        return None;
    }

    // interpret a new price datapoint
    fn new_candlestick(&mut self, candle: &Candlestick){
        self.start_long_trade = self.long_rule.evaluate(candle); 
        self.start_short_trade = self.short_rule.evaluate(candle);
    }

    fn reset(&mut self) {
        self.long_rule.reset();
        self.short_rule.reset();
    }

    fn percentage_amount_per_trade(&self) -> f32 {
        self.percentage_amount_per_trade
    }

    fn leverage(&self) -> u8 {
        self.leverage
    }
}


impl SingleStrategy {
    pub fn decode(cromossome: &[f32]) -> Self{
        if cromossome.len() != 36 {
            panic!("the cromossome must have {} genes, but it had {}", 12, cromossome.len());
        }
        
        for i in 0..cromossome.len() {
            if cromossome[i] < 0.0 || cromossome[i] > 1.0 {
                panic!("the cromossome must have genes between 0.0 and 1.0, but it had {}", cromossome[i]);
            }
        }

        SingleStrategy {
            leverage: map_range((1.0, 60.0), cromossome[0]) as u8,
            long_rule: TradeRule::new(TradeDirection::Long, &cromossome[1..=17]),
            short_rule: TradeRule::new(TradeDirection::Long, &cromossome[18..=35]),
            percentage_amount_per_trade: 0.015,
            start_long_trade: false,
            start_short_trade: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_single_strategy() {
        let strategy = SingleStrategy::decode(vec![0.3;36].as_slice());
        assert_eq!(strategy.leverage, 18);
        assert_eq!(strategy.long_rule.ema_min_percentage_diff_from_price.round(), 30.0);
        assert_eq!(strategy.short_rule.ema_min_percentage_diff_from_price.round(), 30.0);
    }
}