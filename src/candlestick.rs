extern crate csv;
extern crate serde;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Candlestick {
    pub open_time: u64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
    pub close_time: u64,         
    pub quote_asset_volume: f32, 
    pub num_trades: u64,   
}

pub trait Close {
    fn close(&self) -> f32;
}

impl Close for Candlestick {
    fn close(&self) -> f32 {
        self.close
    }
}

pub trait Low {
    fn low(&self) -> f32;
}

impl Low for Candlestick {
    fn low(&self) -> f32 {
        self.low
    }
}

pub trait High {
    fn high(&self) -> f32;
}

impl High for Candlestick {
    fn high(&self) -> f32 {
        self.high
    }
}

impl Candlestick {
    pub fn new() -> Self {
        Self {
            open_time: 0,
            open: 0.0,
            close: 0.0,
            low: 0.0,
            high: 0.0,
            volume: 0.0,
            close_time: 0,
            quote_asset_volume: 0.0,
            num_trades: 0,
        }
    }

    pub fn open(mut self, val: f32) -> Self {
        self.open = val;
        self
    }

    pub fn high(mut self, val: f32) -> Self {
        self.high = val;
        self
    }

    pub fn low(mut self, val: f32) -> Self {
        self.low = val;
        self
    }

    pub fn close(mut self, val: f32) -> Self {
        self.close = val;
        self
    }

    pub fn volume(mut self, val: f32) -> Self {
        self.volume = val;
        self
    }
}

/// load candlestick from a structured csv file
pub fn load_candlesticks(csv_file_path: &str) -> Result<Vec<Candlestick>, csv::Error> {
    let mut reader = csv::Reader::from_path(csv_file_path)?;
    let mut candlesticks: Vec<Candlestick> = Vec::new();

    for result in reader.deserialize() {
        let record: Candlestick = result?;
        candlesticks.push(record);
    }
    Ok(candlesticks)
}