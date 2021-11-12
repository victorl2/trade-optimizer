# TradeOptmizer


## Installation
_(soon)_ You will be able to user Docker and avoid all the manual steps required to setup the development environment if you only want to run **TradeOptimizer**, currently you need **Rust 1.56.0+** _(to run the optimizer)_ and **Python 3.8** _(to generate the dataset)_ installed in your system to properly

### Dataset
You will use a dataset from  the **Bybit Exchange**, a python script is used to connect with the exchange api.
+ Make sure you have **python3+** installed
+ Go inside the folder `dataset_generator`
+ Create a python virtualment and install the required dependencies running `pip3 install -r requirements.txt`
+ run `python download.py` to start downloading the dataset _(it will take a couple of  minutes)_


To download the dataset use  get real exchange data.
The data generate will be a csv with the given format:

| timestamp           | open      | high      | low       | close     | volume     | close_time
| ------------------- |:---------:|:---------:|:---------:|:---------:|:----------:|:------------
| 2019-10-18 20:30:00 | 7922.0700 | 7924.9900 | 7920.1600 | 7924.7500 | 9.90606700 | 1571430659999
| 2019-10-18 20:31:00 | 7923.4300 | 7929.1400 | 7920.8000 | 7922.9000 | 15.83760800| 1571430719999
| 2019-10-18 20:32:00 | 7923.1300 | 7934.0900 | 7922.9000 | 7932.2600 | 9.98577900 | 1571430779999


