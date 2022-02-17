use std::thread::current;

use crate::candlestick;
use candlestick::Candlestick;

pub struct Trade {
    open_timestamp: u64,
    close_timestamp: u64,
    pub side: TradeDirection,
    pub avg_entry_price: f32, // price at the start of the trade
    pub avg_end_price: f32, // price at the end of the trade
    pub leverage: u8, // leverage of the trade
    pub result: f32, // the profit or loss of the trade - all fees are already deducted
    pub initial_position_size: f32, // initial value of the position in USD without leverage
    pub total_fee_paid: f32, // the total fee paid for the trade
    current_position_size: f32, // current value of the position in USD without leverage
    closed_position_size: f32,  // closed value of the position in USD 
    fee_percentage: f32, // the fee percentage charged for the trade
    slipage: f32, // the slipage that was applied to all entry and close prices of the trade
    stoploss: Option<f32>,
    takeprofit: Option<f32>,
    pub liquidation_price: f32,
}

#[derive(PartialEq, Eq)]
pub enum TradeDirection {
    Long,
    Short,
}

impl Trade {
    pub fn open(side: TradeDirection, position_size: f32, start_candle: &Candlestick, leverage: u8, slipage: f32, fee_percentage: f32) -> Trade {
        let mut new_trade = Trade {
            open_timestamp: start_candle.open_time,
            leverage: leverage,
            fee_percentage: fee_percentage,
            slipage: slipage,
            current_position_size: position_size, // current amount of usd of the position
            initial_position_size: position_size, // amount of usd used to start the position
            closed_position_size: 0.0, // amount of usd closed in the trade
            side: side,
            result: 0.0,
            total_fee_paid: 0.0,
            avg_end_price: 0.0,
            close_timestamp: 0,
            avg_entry_price: 0.0,
            stoploss: None,
            takeprofit: None,
            liquidation_price: 0.0,
        };
        new_trade.avg_entry_price = new_trade.slipage_adjusted_price(start_candle.close, true);
        new_trade.apply_transaction_fee(position_size);
        new_trade.liquidation_price = new_trade.liquidation_price();
        new_trade
    }

    pub fn stoploss(&mut self, stoploss: f32) {
        if self.side == TradeDirection::Long && stoploss < self.avg_entry_price {
            self.stoploss = Some(stoploss);
        } else if self.side == TradeDirection::Short && stoploss > self.avg_entry_price {
            self.stoploss = Some(stoploss);
        }
    }

    pub fn takeprofit(&mut self, takeprofit: f32) {
        if self.side == TradeDirection::Long && takeprofit > self.avg_entry_price {
            self.takeprofit = Some(takeprofit);
        } else if self.side == TradeDirection::Short && takeprofit < self.avg_entry_price {
            self.takeprofit = Some(takeprofit);
        }   
    }

    fn liquidation_price(&self) -> f32 {
        let entry = self.avg_entry_price;
        let margin = self.current_position_size * 0.95;
        let total_units = (self.current_position_size * self.leverage as f32 ) / self.avg_entry_price;
        let k = if self.side == TradeDirection::Short{ -1.0 } else { 1.0 };
        (entry * total_units - k * margin) / total_units
    }

    pub fn increase_position(&mut self, current_candle: &Candlestick, position_size_increase: f32) {
        let entry_price_increase = self.slipage_adjusted_price(current_candle.close, true);
        
        let units_already_open = self.current_position_size / self.avg_entry_price;
        let units_to_open = position_size_increase / entry_price_increase;
        self.current_position_size += position_size_increase; 
        self.avg_entry_price = self.current_position_size / (units_already_open + units_to_open);

        self.apply_transaction_fee(position_size_increase);
    }

    pub fn decrease_position(&mut self, price_to_decrease: f32, position_size_decrease: f32) {
        let price_adjusted_decrease = self.slipage_adjusted_price(price_to_decrease, false);
        self.result += self.calculate_result(price_adjusted_decrease, position_size_decrease);
        self.apply_transaction_fee(position_size_decrease);
        
        let units_already_closed = if self.avg_end_price > 0.0{
            self.closed_position_size / self.avg_end_price
        }else {
            0.0
        };

        let units_to_close = position_size_decrease / price_adjusted_decrease;
        
        self.avg_end_price =  (position_size_decrease + self.closed_position_size) / (units_already_closed + units_to_close);

        self.current_position_size -= position_size_decrease;
        self.closed_position_size += position_size_decrease; 
    }

    // returns the profit or loss incurred by the trade
    pub fn close(&mut self, end_candle: &Candlestick) -> f32 {
        self.close_timestamp = end_candle.close_time;
        self.decrease_position(end_candle.close, self.current_position_size);
        self.result
    }

    // returns the loss incurred by the trade
    pub fn close_on_stoploss(&mut self, end_candle: &Candlestick) -> f32{
        if self.stoploss.is_none() {
            return 0.0;
        }
        self.close_timestamp = end_candle.close_time;
        self.decrease_position(self.stoploss.unwrap(), self.current_position_size);
        self.result - self.total_fee_paid
    }

    // returns the profit incurred by the trade
    pub fn close_on_takeprofit(&mut self, end_candle: &Candlestick) -> f32{
        if self.takeprofit.is_none() {
            return 0.0;
        }
        self.close_timestamp = end_candle.close_time;
        self.decrease_position(self.takeprofit.unwrap(), self.current_position_size);
        self.result - self.total_fee_paid
    }

    pub fn close_on_liquidation(&mut self, end_candle: &Candlestick) -> f32{
        self.close_timestamp = end_candle.close_time;
        self.decrease_position(self.liquidation_price, self.current_position_size);
        self.result - self.total_fee_paid
    }

    // amount of minutes the trade was open
    pub fn minutes_open(&self) -> u64{
        (self.close_timestamp - self.open_timestamp) / (60 * 1000) 
    }

    // apply the slipage to the price - 0.5% = 0.005
    // slipage aims to make the price move in the oposite direction of the trade, making it harder to return a profit
    fn slipage_adjusted_price(&self, price: f32, is_position_increase: bool) -> f32 {
        if is_position_increase && self.side == TradeDirection::Long || 
            !is_position_increase && self.side == TradeDirection::Short {
            price * (1.0 + self.slipage)
        } else {
            price * (1.0 - self.slipage)
        }
    }

    fn apply_transaction_fee(&mut self, position_size_transacted: f32) -> f32 {
        let transaction_fee = self.calculate_fee(position_size_transacted);
        self.total_fee_paid += transaction_fee;
        self.result -= transaction_fee;
        transaction_fee
    }
    
    // calculates the profit or loss (without fee) from executing the given transaction at the given price 
    // this function does not update any internal state of the trade
    // the calculation returns the profit in USD ( any profit in crypto is converted to USD)
    fn calculate_result(&self, close_price: f32, position_size_to_close: f32) -> f32 {
        let total_units_to_close = (position_size_to_close * self.leverage as f32) / self.avg_entry_price;
        let initial_value_units = total_units_to_close * self.avg_entry_price;
        let final_value_units = total_units_to_close * close_price;

        if self.side == TradeDirection::Long { 
            final_value_units - initial_value_units
        } else {
            initial_value_units - final_value_units 
        }
    }

    // the fee required to be paid for all trade operations
    // this function does not update any internal state of the trade
    fn calculate_fee(&self, position_size: f32) -> f32 {
        self.fee_percentage * position_size * self.leverage as f32
    }

    pub fn check_profit(&self, current_candle: &Candlestick) -> f32 {
        let adjusted_price = self.slipage_adjusted_price(current_candle.close, false);
        self.calculate_result(adjusted_price, self.closed_position_size)
    }

    pub fn is_takeprofit_reached(&self, current_candle: &Candlestick) -> bool {
        if let Some(takeprofit) = self.takeprofit {
            if self.side == TradeDirection::Long && takeprofit <= current_candle.high ||
                self.side == TradeDirection::Short && takeprofit >= current_candle.low {
                true
            }else{
                false
            }
        } else {
            false
        }
    }

    pub fn is_stoploss_reached(&self, current_candle: &Candlestick) -> bool {
        if let Some(stoploss) = self.stoploss {
            if self.side == TradeDirection::Long && stoploss >= current_candle.low ||
                self.side == TradeDirection::Short && stoploss <= current_candle.high {
                true
            }else{
                false
            }
        } else {
            false
        }
    }

    pub fn is_liquidation_reached(&self, current_candle: &Candlestick) -> bool {
        if self.side == TradeDirection::Long && current_candle.low <= self.liquidation_price ||
            self.side == TradeDirection::Short && current_candle.high >= self.liquidation_price {
            true
        }else{
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_long_trade_correctly() {
        let start_candle = &Candlestick::new().close(100.0);
        let new_trade = Trade::open(TradeDirection::Long, 10.0, start_candle, 20, 0.01, 0.0025);
        assert_eq!(new_trade.avg_entry_price, 101.0);
        assert_eq!(new_trade.total_fee_paid, 0.49999997);
        assert_eq!(new_trade.result, -0.49999997);
    }

    #[test]
    fn open_short_trade_correctly() {
        //side: TradeDirection, units: f32, start_candle: Candlestick, leverage: u8, slipage: f32, fee_percentage:
        let start_candle = &Candlestick::new().close(100.0);
        let new_trade = Trade::open(TradeDirection::Short, 10.0, start_candle, 20, 0.01, 0.0025);
        assert_eq!(new_trade.avg_entry_price, 99.0);
        assert_eq!(new_trade.total_fee_paid, 0.49999997);
        assert_eq!(new_trade.result, -0.49999997);
    }

    #[test]
    fn close_long_trade_correctly() {
        let leverage: u8 = 20;
        let slipage: f32 = 0.01;
        let fee_percent: f32 = 0.0025;
        let units: f32 = 10.0;

        let start_candle = &Candlestick::new().close(300.0);
        let end_candle = &Candlestick::new().close(500.0);

        let mut new_trade = Trade::open(TradeDirection::Long, units, start_candle, leverage, slipage, fee_percent);
        new_trade.close(end_candle);
        assert_eq!(new_trade.avg_end_price, 495.0, "expected avg end price of {} but was {}", 505.0, new_trade.avg_end_price);
        assert_eq!(new_trade.total_fee_paid, 0.99999994, "expected total fee paid of {} but was {}", 399.0, new_trade.total_fee_paid);
        
        assert_eq!(new_trade.result, 125.732666, "expected a profit of {} but was {}", 125.732666, new_trade.result);
    }

    #[test]
    fn close_short_trade_correctly() {
        let leverage: u8 = 20;
        let slipage: f32 = 0.01;
        let fee_percent: f32 = 0.0025;
        let position_size: f32 = 10.0;
        let start_candle = &Candlestick::new().close(300.0);
        
        let end_candle = &Candlestick::new().close(100.0);

        let mut new_trade = Trade::open(TradeDirection::Short, position_size, start_candle, leverage, slipage, fee_percent);
        new_trade.close(end_candle);
        assert_eq!(new_trade.avg_entry_price, 297.0, "expected avg end price of {} but was {}", 297.0, new_trade.avg_entry_price);
        assert_eq!(new_trade.avg_end_price, 101.0, "expected avg end price of {} but was {}", 101.0, new_trade.avg_end_price);
        assert_eq!(new_trade.total_fee_paid, 0.99999994, "expected total fee paid of {} but was {}",199.0, new_trade.total_fee_paid);
        assert_eq!(new_trade.result, 130.98654, "expected a profit of {} but was {}", 130.98654, new_trade.result);
    }

    #[test]
    fn test_long_liquidation_price() {
        let leverage: u8 = 10;
        let position_size: f32 = 200.0;
        let start_candle = &Candlestick::new().close(200.0);

        let new_trade = Trade::open(TradeDirection::Long, position_size, start_candle, leverage, 0.0, 0.0);
        assert_eq!(new_trade.liquidation_price(), 181.0);
    }

    #[test]
    fn test_short_liquidation_price() {
        let leverage: u8 = 10;
        let position_size: f32 = 200.0;
        let start_candle = &Candlestick::new().close(200.0);

        let new_trade = Trade::open(TradeDirection::Short, position_size, start_candle, leverage, 0.0, 0.0);
        assert_eq!(new_trade.liquidation_price(), 219.0);
    }

    #[test]
    fn test_close_trade_with_profit() {
        let leverage: u8 = 10;
        let position_size: f32 = 200.0;
        let start_candle = &Candlestick::new().close(200.0);
        let end_candle = &Candlestick::new().close(400.0);

        let mut new_trade = Trade::open(TradeDirection::Long, position_size, start_candle, leverage, 0.0, 0.0);
        new_trade.close(end_candle);

        assert_eq!(new_trade.result, 2000.0);
    }
}