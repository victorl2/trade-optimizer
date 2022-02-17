use crate::candlestick::{Candlestick, Close};
use crate::indicators::exponential_moving_average::ExponentialMovingAverage as Ema;
use crate::indicators::traits::Next;

use std::fmt;

pub struct RelativeStrengthIndex {
    period: usize,
    up_ema_indicator: Ema,
    down_ema_indicator: Ema,
    prev_val: f32,
    is_new: bool,
    current_rsi: f32,
}

impl RelativeStrengthIndex {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            up_ema_indicator: Ema::new(period),
            down_ema_indicator: Ema::new(period),
            prev_val: 0.0,
            is_new: true,
            current_rsi: 0.0,
        }
    }

    pub fn value(&self) -> f32 {
        self.current_rsi
    }

    pub fn reset(&mut self) {
        self.is_new = true;
        self.prev_val = 0.0;
        self.current_rsi = 0.0;
        self.up_ema_indicator.reset();
        self.down_ema_indicator.reset();
    }
}

impl Next<f32> for RelativeStrengthIndex {
    type Output = f32;

    fn next(&mut self, close_value: f32) -> Self::Output {
        let mut up = 0.0;
        let mut down = 0.0;

        if self.is_new {
            self.is_new = false;
            // Initialize with some small seed numbers to avoid division by zero
            up = 0.1;
            down = 0.1;
        } else {
            if close_value > self.prev_val {
                up = close_value - self.prev_val;
            } else {
                down = self.prev_val - close_value;
            }
        }

        self.prev_val = close_value;
        let up_ema = self.up_ema_indicator.next(up);
        let down_ema = self.down_ema_indicator.next(down);
        self.current_rsi = 100.0 * up_ema / (up_ema + down_ema);
        self.current_rsi
    }
}

impl Next<Candlestick> for RelativeStrengthIndex {
    type Output = f32;

    fn next(&mut self, candle: Candlestick) -> Self::Output {
        self.next(candle.close)
    }
}

impl<T: Close> Next<&T> for RelativeStrengthIndex {
    type Output = f32;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl fmt::Display for RelativeStrengthIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RSI({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rsi = RelativeStrengthIndex::new(10);
        assert_eq!(rsi.period, 10);
    }

    #[test]
    fn test_next() {
        let mut rsi = RelativeStrengthIndex::new(3);
        assert_eq!(rsi.next(&Candlestick::new().close(10.0)), 50.0);
        assert_eq!(rsi.next(&Candlestick::new().close(10.5)).round(), 86.0);
        assert_eq!(rsi.next(&Candlestick::new().close(10.0)).round(), 35.0);
        assert_eq!(rsi.next(&Candlestick::new().close(9.5)).round(), 16.0);
    }

    #[test]
    fn test_display() {
        let rsi = RelativeStrengthIndex::new(16);
        assert_eq!(format!("{}", rsi), "RSI(16)");
    }
}
