"""
@Desc: 对比工具检测结果和benchmark
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

CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
OPEN_SOURCE_PATH = os.path.dirname(CURRENT_PATH)
OPEN_SOURCE_JSON_PATH = os.path.join(OPEN_SOURCE_PATH, "result", "json")


def main():
    # 构建defects结果
    defects_result = {
        "unchecked_return": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "overflow": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "precision_loss": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "infinite_loop": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "constant": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "unused_private_functions": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "unnecessary_type_conversion": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        },
        "unnecessary_bool_judgment": {
            "#Defects": 0,
            "#Benchmark": 0,
            "#TP": 0,
            "#TN": 0,
            "#FP": 0,
            "#FN": 0,
        }
    }

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

        for (module_name,module_info) in package_info["modules"].items():

            # unused_constant
            benchmark_num = module_info["constant"]
            detection_num = len(json_result["modules"][module_name]["detectors"]["unused_constant"])
            defects_result["constant"]["#Benchmark"] += benchmark_num
            defects_result["constant"]["#Defects"] += detection_num
            if detection_num == benchmark_num:
                defects_result["constant"]["#TP"] += benchmark_num
                defects_result["constant"]["#TN"] += json_result["modules"][module_name]["constant_count"] - benchmark_num
            elif detection_num < benchmark_num:
                defects_result["constant"]["#TP"] += detection_num
                defects_result["constant"]["#FN"] += benchmark_num - detection_num
                defects_result["constant"]["#TN"] += json_result["modules"][module_name]["constant_count"] - benchmark_num
            else:
                print("ERROR IN "+json_file)
                print("benchmark("+ str(benchmark_num) + ") < detection(" + str(detection_num) +")") 
            
            # other_defects
            other_defects = ["unchecked_return","overflow","precision_loss","infinite_loop","unnecessary_type_conversion","unnecessary_bool_judgment","unused_private_functions"]
            for defect in other_defects:
                detection_result = {}
                defects_result[defect]["#Defects"] += len(json_result["modules"][module_name]["detectors"][defect]) # 工具检出情况
                for item in json_result["modules"][module_name]["detectors"][defect]: 
                    if defect == "unused_private_functions":
                        func_name = item
                    else: # 去括号，取函数名
                        index = item.find("(")
                        func_name = item[:index]
                    if func_name in detection_result.keys():
                        detection_result[func_name] += 1
                    else:
                        detection_result[func_name] = 1

                for (function_name,benchmark_function_info) in module_info["function"].items():
                    benchmark_defect_cnt = benchmark_function_info[defect]
                    if benchmark_defect_cnt == 0: # benchmark中标注为阴性
                        if function_name in detection_result.keys(): # 但是工具检出了
                            defects_result[defect]["#FP"] += detection_result[function_name]
                            print(package_key+"->"+module_name+"::"+function_name+" --------"+defect+" fail to be marked! * "+str(detection_result[function_name]))
                        else:
                            defects_result[defect]["#TN"] += 1 # 这里看后续如何计数
                    elif benchmark_defect_cnt > 0: # benchmark中标注为阳性
                        defects_result[defect]["#Benchmark"] += benchmark_defect_cnt
                        if function_name in detection_result.keys(): # 工具有检出
                            if detection_result[function_name] == benchmark_defect_cnt: # 工具检出数 == 标记数
                                defects_result[defect]["#TP"] += benchmark_defect_cnt
                            elif detection_result[function_name] < benchmark_defect_cnt: # 工具检出数 < 标记数
                                defects_result[defect]["#TP"] += detection_result[function_name]
                                defects_result[defect]["#FN"] += benchmark_defect_cnt - detection_result[function_name]
                                print(package_key+"->"+module_name+"::"+function_name+" --------"+defect+" fail to be detected! * "+str(benchmark_defect_cnt - detection_result[function_name]))
                            else: # 工具检出数 > 标记数
                                defects_result[defect]["#TP"] += benchmark_defect_cnt
                                defects_result[defect]["#FP"] += detection_result[function_name] - benchmark_defect_cnt
                                print(package_key+"->"+module_name+"::"+function_name+" --------"+defect+" fail to be marked! * "+str(detection_result[function_name] - benchmark_defect_cnt))
                        else: # 但是工具未检出
                            defects_result[defect]["#FN"] += benchmark_defect_cnt
                            print(package_key+"->"+module_name+"::"+function_name+" --------"+defect+" fail to be detected! * "+str(benchmark_defect_cnt))
                    else:
                        print("ERROR IN "+package_key+"->"+module_name+"::"+function_name+"_____"+defect)
                        print("benchmark is less then 0")


    for (key,value) in defects_result.items():
        print(key+str(value))

if __name__ == "__main__":
    main()
