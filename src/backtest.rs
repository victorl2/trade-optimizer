mod trade;
mod trade_rule;
pub mod strategy;

use trade::Trade;
use crate::candlestick;
use candlestick::Candlestick;
use strategy::TradingStrategy;
use crate::utils::split_number_in_points;

use self::strategy::SingleStrategy;

pub struct Backtest {
    candlesticks: Vec<Candlestick>,
    training_ranges: Vec<(u32, u32)>, // ranges of candles in the candlestick vector that will be used for training
    validation_ranges: Vec<(u32, u32)>, // ranges of candles in the candlestick vector that will be used for training
    slipage_percentage: f32, // amount of price change on each trade
    initialization_candles: u32, // number of candles to initialize the strategy with
    fee_percentage: f32, // percentage of the price change that will be charged as fee
    initial_usd_balance: f32, // initial balance in USD
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RunMode {
    Training,
    Validation,
}

impl Backtest {
    
    /// Configures the Backtest engine with **historical price data** and define a **slipage percentage** to emulate price divergence
    /// 
    /// ## Arguments
    /// * candlesticks - historical price data to be used for the backtest
    /// * divisions - number of divisions that define which parts will be used for training and validation
    /// * slipage_percentage - amount of price change on each trade
    /// 
    /// ## Example
    /// ```
    /// mod candlestick;
    /// mod backtest;
    ///  
    /// use backtest::Backtest;
    /// use candlestick::Candlestick;
    /// use candlestick::load_candlesticks;
    /// 
    /// let candlesticks = load_candlesticks("scripts/data_collector/BTCUSDT-5m.csv").unwrap();
    /// let mut backtest = Backtest::new(candlesticks, 12, 0.01);
    /// ```
    pub fn new(candlesticks: Vec<Candlestick>, divisions: u8, slipage_percentage: f32, fee_percentage: f32) -> Backtest {
        let ranges = split_number_in_points(candlesticks.len() as u32, divisions as u32);
        let initial_capacity = (divisions/2) as usize;
        let mut backtest_engine = Backtest {
            candlesticks,
            fee_percentage,
            slipage_percentage,
            initial_usd_balance: 10_000.0,
            initialization_candles: 250,
            training_ranges: Vec::with_capacity(initial_capacity),
            validation_ranges: Vec::with_capacity(initial_capacity),
        };

        for (i, range) in ranges.iter().enumerate() {
            if i % 2 == 0 {
                backtest_engine.training_ranges.push(*range);
            } else {
                backtest_engine.validation_ranges.push(*range);
            }
        }

        backtest_engine  
    }

    // runs a backtest on the provided individual and returns the fitness score
    pub fn run(&self, mode: RunMode, model: &mut SingleStrategy) -> f32 {
        let ranges = if mode == RunMode::Training {
            &self.training_ranges
        } else {
            &self.validation_ranges
        };

        let mut trade_count = 0;
        let mut total_profit: f32 = 0.0;

        //iterating for each separate range
        for range in ranges {
            
            //trade control variables
            let mut balance = self.initial_usd_balance;
            let mut current_trade: Option<Trade> = None;
            let mut current_stoploss: Option<f32> = None;
            let mut current_takeprofit: Option<f32> = None;


            // properly initialize data for internal parameters of the strategy
            for x in range.0..range.0+self.initialization_candles{
                model.new_candlestick(&self.candlesticks[x as usize]);
            }
            
            for i in range.0+self.initialization_candles..range.1 {
                let current_candle = &self.candlesticks[i as usize];
                model.new_candlestick(current_candle);


                if current_trade.is_none() { // there NO trade open
                    let trade_start = model.should_start_trade();
                    if trade_start.is_some() {
                        let trade_start = trade_start.unwrap();
                        let balance_debit = model.percentage_amount_per_trade() * balance;
                        let units_to_trade = balance_debit  / current_candle.close;
                        let mut new_trade = Trade::open(trade_start.0, units_to_trade,
                            current_candle, model.leverage(), self.slipage_percentage, self.fee_percentage);
                                                
                        new_trade.takeprofit(trade_start.1);
                        new_trade.stoploss(trade_start.2);
                        current_trade = Some(new_trade);
                        balance -= balance_debit;
                    }
                }
                // there IS a trade open
                else if current_trade.is_some() { 
                    trade_count += 1;
                    let trade = current_trade.as_mut().unwrap();
                    
                    if trade.is_liquidation_reached(current_candle) {
                        let trade_loss = trade.close_on_liquidation(current_candle);
                        total_profit += trade_loss;
                        balance += trade_loss;
                        current_trade = None;
                    }else if trade.is_stoploss_reached(current_candle) {
                        let trade_loss = trade.close_on_stoploss(current_candle);
                        total_profit += trade_loss;
                        balance += trade_loss;
                        balance -= trade.total_fee_paid;
                        current_trade = None;
                    }else if trade.is_takeprofit_reached(current_candle) {
                        let trade_profit = trade.close_on_takeprofit(current_candle);
                        total_profit += trade_profit;
                        balance += trade_profit;
                        balance -= trade.total_fee_paid;
                        current_trade = None;
                    }                    
                }
            }
            model.reset();
        }

        if trade_count == 0 {
            -self.initial_usd_balance
        }else{
            total_profit
        }
    }
    
    
}

#[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn split_candlesticks_for_validation_and_training(){
            let mut candlesticks = Vec::with_capacity(100);
            for i in 0..100 {
                candlesticks.push(Candlestick::new());
            }

            let backtest_engine = Backtest::new(candlesticks, 20, 0.005, 0.01);
            assert_eq!(backtest_engine.training_ranges.len(), 10);
            assert_eq!(backtest_engine.validation_ranges.len(), 9);
            
            for range in backtest_engine.training_ranges.iter() {
                assert_eq!(range.1 - range.0, 5);
            }

            for range in backtest_engine.validation_ranges.iter() {
                assert_eq!(range.1 - range.0, 5);
            }
        }
    }
