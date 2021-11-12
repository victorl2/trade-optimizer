from datetime import datetime
import pandas as pd
import requests
import time
import os
import json

def to_timestamp(date_string):
    return int(datetime.strptime(date_string, "%d-%m-%Y %H:%M").timestamp())

# converts a timestamp to UTC date in the format %d-%m-%Y %H:%M
def to_utc_date(timestamp):
    return datetime.utcfromtimestamp(int(timestamp)).strftime('%d-%m-%Y %H:%M')

def current_timestamp():
    return int(datetime.now().timestamp())

def progressBar(iterable, prefix = '', suffix = '', decimals = 1, length = 100, fill = '█', printEnd = "\r"):
    """
    Call in a loop to create terminal progress bar
    @params:
        iteration   - Required  : current iteration (Int)
        total       - Required  : total iterations (Int)
        prefix      - Optional  : prefix string (Str)
        suffix      - Optional  : suffix string (Str)
        decimals    - Optional  : positive number of decimals in percent complete (Int)
        length      - Optional  : character length of bar (Int)
        fill        - Optional  : bar fill character (Str)
        printEnd    - Optional  : end character (e.g. "\r", "\r\n") (Str)
    """
    total = len(iterable)
    # Progress Bar Printing Function
    def printProgressBar (iteration):
        percent = ("{0:." + str(decimals) + "f}").format(100 * (iteration / float(total)))
        filledLength = int(length * iteration // total)
        bar = fill * filledLength + '-' * (length - filledLength)
        print(f'\r{prefix} |{bar}| {percent}% {suffix}', end = printEnd)
    # Initial Call
    printProgressBar(0)
    # Update Progress Bar
    for i, item in enumerate(iterable):
        yield item
        printProgressBar(i + 1)
    # Print New Line on Complete
    print()

if __name__ == '__main__':
    start = '01-09-2020 00:00'
    end = None #A valid date or None for the current date

    currency_pair = 'BTCUSD'
    pair_sufix = 'inverse'
    interval = 1 #valid values are: 1,3,5,15,30
    wait_count = 3


    start_date = to_timestamp(start)
    end_date = current_timestamp() if end == None else to_timestamp(end)
    if end == None:
        end = to_utc_date(end_date)
    filename = f'BYBIT-{currency_pair}-{pair_sufix}-{interval}m-data-from-{start.replace(" ", "_")}-to-{end.replace(" ", "_")}.csv'


    print(f'Downloading dataset {currency_pair} {pair_sufix} in candles of {interval}m from {start} to {end}')
    print("total number of candles is",int((int(end_date)-int(start_date))/ ( 60 * interval)))

    list_candle_times = list(range(start_date, end_date, int(200 * 60 * interval)))
    data = []

    for current_time in progressBar(list_candle_times, prefix = 'Progress:', suffix = 'Complete', length = 50):
        status_code = 0
        response = 1

        while(status_code != 200):
            response = requests.get(f'https://api.bybit.com/v2/public/kline/list?symbol={currency_pair}&interval={interval}&limit=200&from={current_time}')
            status_code = response.status_code
            time.sleep(0.1)

        for candle in response.json()["result"]:
            if end_date <= candle["open_time"]:
                break
            data.append([candle["open_time"], candle["open"], candle["high"],candle["low"],candle["close"],candle["volume"], int(candle["open_time"]) + interval * 60])

    df = pd.DataFrame(data, columns=['timestamp', 'open', 'high', 'low', 'close', 'volume', 'close_time'])

    # Saving the CSV
    df.set_index('timestamp', inplace=True)
    df.to_csv(filename, sep=",", encoding='utf-8', index='timestamp')

    print("Download finished !!!")