# Bybit dataset download
Simple script to download historical data from [Bybit](https://www.bybit.com/). The script downloads all candlestick data for a given period and save it in a csv file.

## Usage
Inside the `download.py` file in the first lines after `if __name__ =='__main__:`
you can change the following configuration:

```python
start = '01-03-2019 00:00' # Date in the format day-month-year hour:minute
end = '01-09-2020 00:00' # A valid date or None for the current date
currency_pair = 'BTCUSD' # Currency pair that will be traded
interval = 1 # valid values are: 1,3,5,15,30
```

+ you also need to install the required dependencies running `pip install -r requirements.txt`

Everything is setup!!  you can run the code with `python3 src/download.py`

## Dataset
The data saved in the csv has the given format:

| timestamp           | open      | high      | low       | close     | volume     | close_time
| ------------------- |:---------:|:---------:|:---------:|:---------:|:----------:|:------------
| 2019-10-18 20:30:00 | 7922.0700 | 7924.9900 | 7920.1600 | 7924.7500 | 9.90606700 | 1571430659999
| 2019-10-18 20:31:00 | 7923.4300 | 7929.1400 | 7920.8000 | 7922.9000 | 15.83760800| 1571430719999
| 2019-10-18 20:32:00 | 7923.1300 | 7934.0900 | 7922.9000 | 7932.2600 | 9.98577900 | 1571430779999
