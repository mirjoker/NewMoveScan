"""
@Desc: 工具性能分析
@Author: Yule
"""
from loguru import logger
import os
import re
import subprocess
import concurrent
import json
from enum import Enum
from openpyxl import Workbook
from openpyxl.styles import Font, Alignment
import csv

CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
OPEN_SOURCE_PATH = os.path.dirname(CURRENT_PATH)
OPEN_SOURCE_JSON_PATH = os.path.join(OPEN_SOURCE_PATH, "result", "json")


def main():
    # 构建defects结果
    time_result = {}

    # 读入benchmark
    benchmark_file = os.path.join(OPEN_SOURCE_PATH, "result", "OpenSource_benchmark.json")
    with open(benchmark_file, "r") as file:
        benchmark = json.load(file)
    for (package_key,package_info) in benchmark.items():
        if package_info["chain_type"] == 0:
            json_path = os.path.join(OPEN_SOURCE_JSON_PATH, "Aptos", package_key)
        else:
            json_path = os.path.join(OPEN_SOURCE_JSON_PATH, "Sui", package_key)
        for item in os.listdir(json_path):
            json_file = os.path.join(json_path,item)
            if os.path.isfile(json_file):
                with open(json_file, "r") as file:
                    json_result = json.load(file)
                break
        time_result[package_key] = json_result["total_time"]
    
    for (key,value) in time_result.items():
        print(key+":"+str(value))

    csv_file =  os.path.join(CURRENT_PATH, 'time_analysis.csv')

    # 将字典数据写入CSV文件
    with open(csv_file, mode='w', newline='') as file:
        fieldnames = ['package_name', 'time']  # 列名
        writer = csv.writer(file)
        writer.writerow(fieldnames)
        # 写入字典数据
        for (key,value) in time_result.items():
            writer.writerow([key,value])

if __name__ == "__main__":
    main()
