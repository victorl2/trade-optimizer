use crate::candlestick::{Close, High, Low};
use crate::indicators::traits::{Next, Period};
use crate::indicators::exponential_moving_average::ExponentialMovingAverage as Ema;
use crate::indicators::true_range::TrueRange;
use std::fmt;

pub struct AverageTrueRange {
    true_range: TrueRange,
    ema: Ema,
}

impl AverageTrueRange {
    pub fn new(period: usize) -> Self {
        Self {
            true_range: TrueRange::new(),
            ema: Ema::new(period),
        }
    }

    pub fn value(&self) -> f32 {
        self.ema.value()
    }

    pub fn reset(&mut self) {
        self.true_range.reset();
        self.ema.reset();
    }
}

impl Period for AverageTrueRange {
    fn period(&self) -> usize {
        self.ema.period()
    }
}

impl Next<f32> for AverageTrueRange {
    type Output = f32;

    fn next(&mut self, input: f32) -> Self::Output {
        self.ema.next(self.true_range.next(input))
    }
}

impl<T: High + Low + Close> Next<&T> for AverageTrueRange {
    type Output = f32;

    fn next(&mut self, input: &T) -> Self::Output {
        self.ema.next(self.true_range.next(input))
    }
}

impl fmt::Display for AverageTrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ATR({})", self.ema.period())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candlestick::Candlestick;

    #[test]
    fn test_new() {
        let atr = AverageTrueRange::new(10);
        assert_eq!(atr.period(), 10);
    }
    #[test]
    fn test_next() {
        let mut atr = AverageTrueRange::new(3);

        let candle1 = Candlestick::new().high(10.0).low(7.5).close(9.0);
        let candle2 = Candlestick::new().high(11.0).low(9.0).close(9.5);
        let candle3 = Candlestick::new().high(9.0).low(5.0).close(8.0);

        assert_eq!(atr.next(&candle1), 2.5);
        assert_eq!(atr.next(&candle2), 2.25);
        assert_eq!(atr.next(&candle3), 3.375);
    }

    #[test]
    fn test_display() {
        let indicator = AverageTrueRange::new(8);
        assert_eq!(format!("{}", indicator), "ATR(8)");
    }
}
