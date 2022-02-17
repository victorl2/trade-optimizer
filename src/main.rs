
pub mod candlestick;
pub mod indicators;
pub mod backtest;
pub mod brkga;
mod utils;

use candlestick::Candlestick;
use backtest::{Backtest, RunMode};
use brkga::{BRKGA, BrkgaConfig, FitnessExecutor};

fn main() {
    let csv_path = "scripts/data_collector/ETHUSDT-5m.csv";
    
    let candles = match candlestick::load_candlesticks(csv_path) {
        Ok(candles) => candles,
        Err(_) => {
            println!("Could't load the file {}. Check if the file exists and has the required csv structure.", csv_path);
            return;
        }
    };
    println!("found {} candles inside {}", candles.len(), csv_path);

    let seed: u64 = 18988547;
    let frac_bot: f32 = 0.3;
    let frac_top: f32 = 0.1;
    let pop_size : usize = 10000;
    let max_iter: usize = 100;
    let elit_rate : f32 = 0.6;

    run_experiment(candles, seed, (frac_top, frac_bot, pop_size,  max_iter, elit_rate));
}

fn run_experiment(candles: Vec<Candlestick>, seed: u64, config: BrkgaConfig) {
    println!("Running backtest with {} divisions", 12);
    
    let backtest_engine = Backtest::new(candles, 12, 0.005, 0.02);
    let mut brkga = BRKGA::new(seed, 36, config, 
        FitnessExecutor::new(backtest_engine, RunMode::Training));
    brkga.run();
}