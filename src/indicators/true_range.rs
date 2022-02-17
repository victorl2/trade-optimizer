use crate::candlestick::{Candlestick, Close, High, Low};
use crate::indicators::traits::{ Next };
use std::fmt;

pub struct TrueRange {
    prev_close: Option<f32>,
}

impl TrueRange {
    pub fn new() -> Self {
        Self { prev_close: None }
    }

    pub fn reset(&mut self) {
        self.prev_close = None;
    }
}

impl fmt::Display for TrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TRUE_RANGE()")
    }
}

impl Next<f32> for TrueRange {
    type Output = f32;

    fn next(&mut self, input: f32) -> Self::Output {
        let distance = match self.prev_close {
            Some(prev) => (input - prev).abs(),
            None => 0.0,
        };
        self.prev_close = Some(input);
        distance
    }
}

impl<T: High + Low + Close> Next<&T> for TrueRange {
    type Output = f32;

    fn next(&mut self, candle: &T) -> Self::Output {
        let max_dist = match self.prev_close {
            Some(prev_close) => {
                let dist1 = candle.high() - candle.low();
                let dist2 = (candle.high() - prev_close).abs();
                let dist3 = (candle.low() - prev_close).abs();
                dist1.max(dist2).max(dist3)
            }
            None => candle.high() - candle.low(),
        };
        self.prev_close = Some(candle.close());
        max_dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_f32() {
        let mut tr = TrueRange::new();
        assert_eq!(tr.next(2.5), 0.0);
        assert_eq!(tr.next(3.6), 1.0999999);
        assert_eq!(tr.next(3.3), 0.29999995);
    }

    #[test]
    fn test_next_candle() {
        let mut tr = TrueRange::new();

        let candle1 = Candlestick::new().high(10.0).low(7.5).close(9.0);
        let candle2 = Candlestick::new().high(11.0).low(9.0).close(9.5);
        let candle3 = Candlestick::new().high(9.0).low(5.0).close(8.0);

        assert_eq!(tr.next(&candle1), 2.5);
        assert_eq!(tr.next(&candle2), 2.0);
        assert_eq!(tr.next(&candle3), 4.5);
    }

    #[test]
    fn test_display() {
        let indicator = TrueRange::new();
        assert_eq!(format!("{}", indicator), "TRUE_RANGE()");
    }
}



