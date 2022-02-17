use std::f32;

use crate::candlestick::Candlestick;
use crate::backtest::trade::TradeDirection;
use crate::utils::{map_range, percentage_difference};

use crate::indicators::traits::Next;
use crate::indicators::simple_moving_average::SimpleMovingAverage as Sma;
use crate::indicators::average_true_range::AverageTrueRange as ATR;
use crate::indicators::relative_strength_index::RelativeStrengthIndex as Rsi;
use crate::indicators::exponential_moving_average::ExponentialMovingAverage as Ema;
use crate::indicators::moving_average_convergence_divergence::MovingAverageConvergenceDivergence as Macd;

pub struct TradeRule {
    direction: TradeDirection,
    sma: Sma,
    rsi: Rsi,
    macd: Macd,
    ema: Ema,
    take_profit_atr: ATR,
    stoploss_atr: ATR,
    
    macd_target_value: f32,
    rsi_lower_bound: f32,
    rsi_higher_bound: f32,
    pub ema_min_percentage_diff_from_price: f32,
    pub sma_max_percentage_diff_from_ema: f32,

    take_profit_target_atr: f32,
    stop_loss_target_atr: f32,

    ema_takeprofit: Sma,
    ema_stoploss: Sma,
}

impl TradeRule {
    pub fn reset(&mut self) {
        self.sma.reset();
        self.rsi.reset();
        self.macd.reset();
        self.ema.reset();
        self.take_profit_atr.reset();
        self.stoploss_atr.reset();
        self.ema_takeprofit.reset();
        self.ema_stoploss.reset();
    }

    pub fn new(direction: TradeDirection, cromossome: &[f32]) -> Self {
        let macd_period1 = map_range((2.0, 100.0), cromossome[7]) as usize;
        let macd_period2 = map_range((2.0, 100.0), cromossome[8]) as usize;
        let macd_singal_period = map_range((2.0, 100.0), cromossome[9]) as usize;

        let rsi_bound1 = map_range((0.0, 100.0), cromossome[5]);
        let rsi_bound2 = map_range((0.0, 100.0), cromossome[6]);

        TradeRule {
            direction: direction,
            take_profit_target_atr: map_range( (0.1, 20.0),cromossome[0]),  
            take_profit_atr: ATR::new(map_range((2.0, 100.0), cromossome[1]) as usize),
            stop_loss_target_atr: map_range( (0.1, 20.0),cromossome[2]),
            stoploss_atr: ATR::new(map_range((2.0, 50.0),cromossome[3]) as usize),
            rsi: Rsi::new(map_range((0.0, 100.0), cromossome[4]) as usize),
            rsi_lower_bound: rsi_bound1.max(rsi_bound2),
            rsi_higher_bound: rsi_bound1.min(rsi_bound2),
            macd: Macd::new(macd_period1, macd_period2, macd_singal_period),
            macd_target_value: map_range((-1000.0, 1000.0), cromossome[10]),
            ema: Ema::new(map_range((2.0, 100.0), cromossome[11]) as usize),
            ema_min_percentage_diff_from_price: map_range((0.1, 100.0), cromossome[12]),
            sma: Sma::new(map_range((2.0, 100.0), cromossome[13]) as usize),
            sma_max_percentage_diff_from_ema: map_range((0.1, 100.0), cromossome[14]),
            ema_takeprofit: Sma::new(map_range((1.0, 100.0), cromossome[15]) as usize),
            ema_stoploss: Sma::new(map_range((1.0, 100.0), cromossome[16]) as usize),
        } 
    }

    pub fn evaluate(&mut self, candle: &Candlestick) -> bool {
        let rsi = self.rsi.next(candle.close);
        let macd = self.macd.next(candle);
        let ema = self.ema.next(candle);
        let sma = self.sma.next(candle);
        let pd_ema_from_close = percentage_difference(ema, candle.close);
        let pd_sma_from_ema = percentage_difference(sma, ema);

        self.ema_takeprofit.next(if self.direction == TradeDirection::Long { candle.high } else { candle.low });
        self.ema_stoploss.next(if self.direction == TradeDirection::Long { candle.low } else { candle.high });

        self.take_profit_atr.next(candle);
        self.stoploss_atr.next(candle);
        
        //expression that evaluates if a trade should be opened
        macd.signal > self.macd_target_value &&
        rsi > self.rsi_higher_bound && rsi < self.rsi_lower_bound &&
        pd_ema_from_close >= self.ema_min_percentage_diff_from_price ||
        pd_sma_from_ema <= self.sma_max_percentage_diff_from_ema
    }

    pub fn evaluate_take_profit(&self) -> f32 {
        let diff = self.take_profit_atr.value() * self.take_profit_target_atr;
        if self.direction == TradeDirection::Long {
            self.ema_takeprofit.max_value_on_period() + diff
        }else{
            self.ema_takeprofit.min_value_on_period() - diff
        }
    }

    pub fn evaluate_stop_loss(&self) -> f32 {
        let diff = self.stoploss_atr.value() * self.stop_loss_target_atr;
        if self.direction == TradeDirection::Long {
            self.ema_stoploss.min_value_on_period() - diff
        }else{
            self.ema_stoploss.max_value_on_period() + diff
        }
    }
}