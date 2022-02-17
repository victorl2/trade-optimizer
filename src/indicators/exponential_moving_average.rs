use crate::candlestick::{Candlestick, Close};
use crate::indicators::traits::Next;
use std::fmt;

pub struct ExponentialMovingAverage {
    period: usize,
    k: f32,
    current: f32,
    is_new: bool,
}

impl ExponentialMovingAverage {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            k: 2.0 / (period + 1) as f32,
            current: 0.0,
            is_new: true,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn reset(&mut self) {
        self.current = 0.0;
        self.is_new = true;
    }
}

impl Next<f32> for ExponentialMovingAverage {
    type Output = f32;

    fn next(&mut self, close_value: f32) -> Self::Output {
        if self.is_new {
            self.is_new = false;
            self.current = close_value;
        } else {
            self.current = self.k * close_value + (1.0 - self.k) * self.current;
        }
        self.current
    }
}

impl Next<Candlestick> for ExponentialMovingAverage {
    type Output = f32;

    fn next(&mut self, candle: Candlestick) -> Self::Output {
        self.next(candle.close)
    }
}

impl<T: Close> Next<&T> for ExponentialMovingAverage {
    type Output = f32;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl fmt::Display for ExponentialMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ema = ExponentialMovingAverage::new(5);
        assert_eq!(ema.period, 5);

        let ema = ExponentialMovingAverage::new(0);
        assert_eq!(ema.period, 0);
    }

    #[test]
    fn test_next() {
        let mut ema = ExponentialMovingAverage::new(3);

        assert_eq!(ema.next(&Candlestick::new().close(2.0)), 2.0);
        assert_eq!(ema.next(&Candlestick::new().close(5.0)), 3.5);
        assert_eq!(ema.next(&Candlestick::new().close(1.0)), 2.25);
        assert_eq!(ema.next(&Candlestick::new().close(6.25)), 4.25);

        let mut ema = ExponentialMovingAverage::new(3);
        let candle1 = Candlestick::new().close(2.0);
        let candle2 = Candlestick::new().close(5.0);
        assert_eq!(ema.next(&candle1), 2.0);
        assert_eq!(ema.next(&candle2), 3.5);
    }

    #[test]
    fn test_display() {
        let ema = ExponentialMovingAverage::new(7);
        assert_eq!(format!("{}", ema), "EMA(7)");
    }
}
