use crate::candlestick::{Candlestick, Close};
use crate::indicators::traits::Next;
use crate::indicators::exponential_moving_average::ExponentialMovingAverage as Ema;
use std::fmt;

pub struct MovingAverageConvergenceDivergence {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    last_signal: f32,
}

impl MovingAverageConvergenceDivergence {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
            last_signal: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.last_signal = 0.0;
    }

    pub fn value(&self) -> f32 {
        self.last_signal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovingAverageConvergenceDivergenceOutput {
    pub macd: f32,
    pub signal: f32,
    pub histogram: f32,
}

impl From<MovingAverageConvergenceDivergenceOutput> for (f32, f32, f32) {
    fn from(mo: MovingAverageConvergenceDivergenceOutput) -> Self {
        (mo.macd, mo.signal, mo.histogram)
    }
}

impl Next<f32> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: f32) -> Self::Output {
        let fast_val = self.fast_ema.next(input);
        let slow_val = self.slow_ema.next(input);

        let macd = fast_val - slow_val;
        let signal = self.signal_ema.next(macd);
        let histogram = macd - signal;

        self.last_signal = signal;

        MovingAverageConvergenceDivergenceOutput {
            macd,
            signal,
            histogram,
        }
    }
}

impl Next<Candlestick> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, candle: Candlestick) -> Self::Output {
        self.next(candle.close)
    }
}

impl<T: Close> Next<&T> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl fmt::Display for MovingAverageConvergenceDivergence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MACD({}, {}, {})",
            self.fast_ema.period(),
            self.slow_ema.period(),
            self.signal_ema.period()
        )
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    type Macd = MovingAverageConvergenceDivergence;

    fn round(nums: (f32, f32, f32)) -> (f32, f32, f32) {
        let n0 = (nums.0 * 100.0).round() / 100.0;
        let n1 = (nums.1 * 100.0).round() / 100.0;
        let n2 = (nums.2 * 100.0).round() / 100.0;
        (n0, n1, n2)
    }

    #[test]
    fn test_macd() {
        let mut macd = Macd::new(3, 6, 4);

        assert_eq!(round(macd.next(2.0).into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).into()), (0.21, 0.09, 0.13));
        assert_eq!(round(macd.next(4.2).into()), (0.52, 0.26, 0.26));
        assert_eq!(round(macd.next(7.0).into()), (1.15, 0.62, 0.54));
        assert_eq!(round(macd.next(6.7).into()), (1.15, 0.83, 0.32));
        assert_eq!(round(macd.next(6.5).into()), (0.94, 0.87, 0.07));
    }
    
    #[test]
    fn test_display() {
        let indicator = Macd::new(13, 30, 10);
        assert_eq!(format!("{}", indicator), "MACD(13, 30, 10)");
    }
}