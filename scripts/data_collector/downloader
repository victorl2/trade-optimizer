#!/usr/bin/env python3

import requests
import json
import time
import csv
import argparse

#if the returned status code is 429 apply a exponential backoff

# parameters used to download data from Binance futures
# more information on: https://binance-docs.github.io/apidocs/futures/en
intervals = ['1m', '3m', '5m', '15m', '30m', '1h', '2h', '4h', '6h', '8h', '12h', '1d', '3d', '1w', '1M']
base_url = 'https://fapi.binance.com/fapi/v1'
seconds_to_wait = 1 # default amount of seconds to wait the the server returns a status code of 429

def backoff(response: requests.Response) -> bool:
    """
    exponential backoff to allow the server to process when the server limit is reached
    :param response: response from the server
    :return: True if the server limit is reached, False otherwise

    if True is returned the backoff was applied and the request should be retried
    if False is returned the request was sucessfull and should NOT be retried
    """
    global seconds_to_wait

    if response is None:
        return True

    if response.status_code == 429:
        print(f'server limit reached, waiting {seconds_to_wait} seconds')
        time.sleep(seconds_to_wait)
        seconds_to_wait *= 2
        return True
    else:
        seconds_to_wait = 1
        return False

def server_time() -> int:
    """ test the connection with the server and return the current server time"""
    response = requests.get(f"{base_url}/time") 
    if response.status_code != 200:
        raise Exception(f"Error connecting to Binance. status code {response.status_code}")
    return json.loads(response.text)['serverTime']



def kline(pair: str, interval: int, start_time: str, limit: int) -> list:
    """
    get candlestick price data from Binance Futures

    :param pair: string of cryptocurrency pair
    :param interval: string of interval between candlesticks
    :param start_time: string of start date
    :param limit: int of number of candlesticks to return
    :return: list of candlesticks
    """
    url = f'{base_url}/klines?symbol={pair}&interval={interval}&limit={limit}&startTime={start_time}'
    response = None
    while(backoff(response)):
        try:
            response = requests.get(url)
        except:
            response = None
            print('network error, retrying connection to Binance')
    data = json.loads(response.text)
    return data

def prices_to_csv(symbol: str, interval: int, start_time: str):
    """
    construct a csv file with the price data from Binance Futures with candlestick data for the symbol in the interval
    :param pair: string of cryptocurrency pair
    :param start_date: string of start date
    :return: dataframe of price data
    """
    if interval not in intervals:
        raise ValueError(f'interval {interval} not supported')
    kline_columns = ['OpenTime', 'Open', 'High', 'Low', 'Close', 'Volume', 'CloseTime', 'QuoteAssetVolume', 'NumTrades']
    candles_per_request = 499
    current_time = start_time

    with open(f'{symbol}-{interval}.csv', 'w') as csvfile:
        csvwriter = csv.writer(csvfile)
        csvwriter.writerow(kline_columns)
    
        while(True):
            price_data = kline(symbol, interval, current_time, candles_per_request)
            for row in price_data:
                csvwriter.writerow(row[:9])
            if len(price_data) < 2:
                break
            current_time = price_data[len(price_data)-1][6] # close time of the last candle returned
            print('.', end='', flush=True)

    print(f'\n>finished downloading candlestick data for {symbol} in {interval}')

def cli_interface() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description='Welcome to candlestick data downloader for Binance Futures', 
        prog='downloader', usage='./downloader BTCUSDT -interval=30m -days=365\n>>for more details run ./downloader -h')
    parser.add_argument('symbol', help='symbol of the desired cryptocurrency pair', type=str)
    parser.add_argument('-interval', help='interval between candlesticks', type=str, choices=intervals, default='1m', required=False)
    parser.add_argument('-days', help='number of days to collect data going back from today', type=int, default=1095, required=False)
    return parser.parse_args()

if __name__ == '__main__':
    parsed_values = cli_interface()
        
    symbol = parsed_values.symbol
    interval = parsed_values.interval
    days = parsed_values.days

    current_time = server_time()
    offset = 60*60*24*days*1000 # time offset in unix milis
    start_timestamp = str(int(current_time) - offset)
    print(f'>downloading {symbol} with candles of {interval} from {start_timestamp} until {current_time}')

    # download and save the candlestick price to a csv file
    prices_to_csv(symbol, interval, start_timestamp)