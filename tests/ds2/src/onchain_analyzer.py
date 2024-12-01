from loguru import logger
import os
import re
import subprocess
import concurrent
import json
from enum import Enum

CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ONCHAIN_PATH = os.path.dirname(CURRENT_PATH)
ONCHAIN_BYTECODE_PATH = os.path.join(ONCHAIN_PATH, "res", "bytecode")
ONCHAIN_RESULT_PATH = os.path.join(ONCHAIN_PATH, "result")

LOG_PATH = os.path.join(CURRENT_PATH, "onchain_analyzer.log")
logger.add(LOG_PATH, format="{time} | {level}    | {message}", level="WARNING")


def run_movescanner(command):
    sp = subprocess.Popen(command)
    retrun_code = sp.wait()
    if retrun_code != 0:
        logger.error("return code: {}, command: {}", retrun_code, command)


def main():
    logger.info("Start Aptos")
    commands = []
    aptos_bytecode_path = os.path.join(ONCHAIN_BYTECODE_PATH, "Aptos")
    aptos_json_path = os.path.join(ONCHAIN_RESULT_PATH, "json", "Aptos")
    for root, _dirs, files in os.walk(aptos_bytecode_path):
        for file in files:
            file_path = os.path.join(root, file)
            output_filename = os.path.splitext(file)[0] + ".json"
            output_path = os.path.join(aptos_json_path, output_filename)
            command = ["MoveScanner", "-p", file_path, "-o", output_path, "-n"]
            commands.append(command)
    logger.info("MoveScanner running...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=20) as executor:
        executor.map(run_movescanner, commands)
    logger.success("Aptos Done")

    logger.info("Start Sui")
    commands = []
    sui_bytecode_path = os.path.join(ONCHAIN_BYTECODE_PATH, "Sui")
    sui_json_path = os.path.join(ONCHAIN_RESULT_PATH, "json", "Sui")
    for root, _dirs, files in os.walk(sui_bytecode_path):
        for file in files:
            file_path = os.path.join(root, file)
            output_filename = os.path.splitext(file)[0] + ".json"
            output_path = os.path.join(sui_json_path, output_filename)
            command = ["MoveScanner", "-p", file_path, "-o", output_path, "-n"]
            commands.append(command)
    logger.info("MoveScanner running...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=20) as executor:
        executor.map(run_movescanner, commands)
    logger.success("Sui Done")


# 分析执行结果，获取绘图/表所需数据集
DATA_TYPE = Enum(
    "DATA_TYPE",
    (
        "FileName_to_Time",
        "FileName_to_Size",
        "Total_Time",
        "Module_Count",
        "Constant_Count",
        "Function_Count",
        "Detectors_to_Count",
    ),
)


def onchain_analyzer(data_type: DATA_TYPE):
    result_path = os.path.join(ONCHAIN_PATH, "result", "json")
    if data_type == DATA_TYPE.FileName_to_Time:
        filename_to_time = {"All": {}, "Sui": {}, "Aptos": {}}
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    time = json_data.get("total_time")
                    if filename not in filename_to_time.get(chian_type):
                        filename_to_time.get(chian_type)[filename] = time
                    else:
                        raise
                    if filename not in filename_to_time.get("All"):
                        filename_to_time.get("All")[filename] = time
                    else:
                        raise
        # logger.info("filename_to_time: {}".format(filename_to_time))
        return filename_to_time
    elif data_type == DATA_TYPE.FileName_to_Size:
        filename_to_size = {"All": {}, "Sui": {}, "Aptos": {}}
        ONCHAIN_BYTECODE_PATH = os.path.join(ONCHAIN_PATH, "res", "bytecode")
        for root, _dirs, files in os.walk(ONCHAIN_BYTECODE_PATH):
            for file in files:
                if not file.endswith(".mv"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/bytecode/(.*?)/(.*?)\.mv"
                chian_type, filename = re.findall(pattern, file_path)[0]
                file_size = os.path.getsize(file_path)
                filename_to_size.get(chian_type)[filename] = file_size
                filename_to_size.get("All")[filename] = file_size
        # logger.info("filename_to_size: {}".format(filename_to_size))
        return filename_to_size
    elif data_type == DATA_TYPE.Total_Time:
        total_time = {"Sui": 0, "Aptos": 0, "All": 0}
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    time = json_data.get("total_time")
                    total_time[chian_type] += time
                    total_time["All"] += time
        logger.info("total_time: {}".format(total_time))
        return total_time
    elif data_type == DATA_TYPE.Module_Count:
        module_count = {"Sui": 0, "Aptos": 0, "All": 0}
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    module_count[chian_type] += json_data.get("modules_status").get(
                        "wrong"
                    )
                    module_count[chian_type] += json_data.get("modules_status").get(
                        "pass"
                    )
                    module_count["All"] += json_data.get("modules_status").get("wrong")
                    module_count["All"] += json_data.get("modules_status").get("pass")
        logger.info("module_count: {}".format(module_count))
        return module_count
    elif data_type == DATA_TYPE.Constant_Count:
        constant_count = {"Sui": 0, "Aptos": 0, "All": 0}
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name, module in json_data.get("modules").items():
                        constant_count[chian_type] += module.get("constant_count")
                        constant_count["All"] += module.get("constant_count")
        logger.info("constant_count: {}".format(constant_count))
        return constant_count
    elif data_type == DATA_TYPE.Function_Count:
        function_count = {
            "Sui": {"all": 0, "native": 0},
            "Aptos": {"all": 0, "native": 0},
            "All": {"all": 0, "native": 0},
        }
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name, module in json_data.get("modules").items():
                        function_count[chian_type]["all"] += module.get(
                            "function_count"
                        ).get("all")
                        function_count[chian_type]["native"] += module.get(
                            "function_count"
                        ).get("native")
                        function_count["All"]["all"] += module.get(
                            "function_count"
                        ).get("all")
                        function_count["All"]["native"] += module.get(
                            "function_count"
                        ).get("native")
        logger.info("function_count: {}".format(function_count))
        return function_count
    elif data_type == DATA_TYPE.Detectors_to_Count:
        detectors_to_count = {
            "Sui": {
                "precision_loss": 0,
                "infinite_loop": 0,
                "unnecessary_type_conversion": 0,
                "overflow": 0,
                "unchecked_return": 0,
                "unused_constant": 0,
                "unnecessary_bool_judgment": 0,
                "unused_private_functions": 0,
            },
            "Aptos": {
                "precision_loss": 0,
                "infinite_loop": 0,
                "unnecessary_type_conversion": 0,
                "overflow": 0,
                "unchecked_return": 0,
                "unused_constant": 0,
                "unnecessary_bool_judgment": 0,
                "unused_private_functions": 0,
            },
            "All": {
                "precision_loss": 0,
                "infinite_loop": 0,
                "unnecessary_type_conversion": 0,
                "overflow": 0,
                "unchecked_return": 0,
                "unused_constant": 0,
                "unnecessary_bool_judgment": 0,
                "unused_private_functions": 0,
            },
        }
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith(".json"):
                    continue
                file_path = os.path.join(root, file)
                pattern = r"/result/json/(.*?)/(.*?).json"
                chian_type, filename = re.findall(pattern, file_path)[0]
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name, module in json_data.get("modules").items():
                        for detector_name, detector_result in module.get(
                            "detectors"
                        ).items():
                            detectors_to_count[chian_type][detector_name] += len(
                                detector_result
                            )
                            detectors_to_count["All"][detector_name] += len(
                                detector_result
                            )
        logger.info("detectors_to_count: {}".format(detectors_to_count))
        return detectors_to_count


if __name__ == "__main__":
    main()
