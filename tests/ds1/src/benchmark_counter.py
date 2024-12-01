import os
import json

CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ROOT_PATH = os.path.dirname(os.path.dirname(CURRENT_PATH))
OPEN_SOURCE_JSON_PATH = os.path.join(ROOT_PATH, "OpenSource", "result", "OpenSource_benchmark.json")
ORIGN_JSON_PATH = os.path.join(ROOT_PATH, "MoveLint", "result", "Origin_benchmark.json")
DETECT_LOCATION_PATH = os.path.join(CURRENT_PATH, "detect_locations.json")

result = {"Aptos": 0,
          "Sui": 0,
          "Move": 0,
          "Total": 0
          }

def detect_counter():
    with open(OPEN_SOURCE_JSON_PATH, 'r') as file:
        json_reader = json.load(file)
        for (package_key, package_info) in json_reader.items():
            blockchain_id = "Aptos" if package_info["chain_type"] == 0 else "Sui"
            for (module_name, module_info) in package_info["modules"].items():
                flag = 0
                for (function_name, function_info) in module_info["function"].items():
                    for (detecter, num) in function_info.items():
                        if num != 0:
                            flag = 1
                            break
                if module_info["constant"] != 0:
                    flag = 1
                result[blockchain_id] += flag
                result["Total"] += flag

    with open(ORIGN_JSON_PATH, 'r') as file:
        json_reader = json.load(file)
        for (package_key, package_info) in json_reader.items():
            blockchain_id = "Move"
            for (module_name, module_info) in package_info["modules"].items():
                flag = 0
                for (function_name, function_info) in module_info["function"].items():
                    for (detecter, num) in function_info.items():
                        if num != 0:
                            flag = 1
                            break
                if module_info["constant"] != 0:
                    flag = 1
                result[blockchain_id] += flag
                result["Total"] += flag

def module_counter():
    with open(OPEN_SOURCE_JSON_PATH, 'r') as file:
        json_reader = json.load(file)
        for (package_key, package_info) in json_reader.items():
            blockchain_id = "Aptos" if package_info["chain_type"] == 0 else "Sui"
            for (module_name, module_info) in package_info["modules"].items():
                result[blockchain_id] += 1
                result["Total"] += 1

    with open(ORIGN_JSON_PATH, 'r') as file:
        json_reader = json.load(file)
        for (package_key, package_info) in json_reader.items():
            blockchain_id = "Move"
            for (module_name, module_info) in package_info["modules"].items():
                result[blockchain_id] += 1
                result["Total"] += 1

if __name__ == "__main__":
    module_counter()
    print(result)