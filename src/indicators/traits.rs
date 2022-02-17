// the time-period used for the indicator
pub trait Period {
    fn period(&self) -> usize;
}

/// Next describes the hability of continually evaluate a given indicator. It returns the next indicator when given a new candlestick
/// for most indicators `Output` is a f64, but it can be a more complex type.
pub trait Next<T> { 
    type Output;
    fn next(&mut self, input: T) -> Self::Output;
}
