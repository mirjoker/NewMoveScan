"""
@Desc: 使用 MoveScanner 处理所有的开源项目
@Author: HappyTsing
"""
from loguru import logger
import os
import re
import subprocess
import concurrent
import json
from enum import Enum
CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
OPEN_SOURCE_PATH = os.path.dirname(CURRENT_PATH)
OPEN_SOURCE_REPO_PATH = os.path.join(OPEN_SOURCE_PATH,"res","repo")

LOG_PATH = os.path.join(CURRENT_PATH, "open_source_analyzer.log")
logger.add(LOG_PATH, format='{time} | {level}    | {message}', level="WARNING")

def create_dirs(folders_path):
    for folder_path in folders_path:
        if not os.path.exists(folder_path):
            os.makedirs(folder_path)
        #     logger.info(f"Folder '{folder_path}' created successfully.")
        # else:
        #     logger.info(f"Folder '{folder_path}' already exists.")

def run_movescanner(command):
    sp = subprocess.Popen(command)
    logger.info("当前执行：{}",command)
    retrun_code = sp.wait()
    if retrun_code != 0:
        logger.error("return code: {}, command: {}",retrun_code,command)
    else:
        logger.info("执行完毕：{}",command)


        
# 使用 MoveScanner 执行所有开源项目
def main():
    result_path = os.path.join(OPEN_SOURCE_PATH,"result","json")
    result_aptos_path = os.path.join(result_path,"Aptos")
    result_sui_path = os.path.join(result_path,"Sui")
    create_dirs([result_path,result_aptos_path,result_sui_path])
    # result_move_path = os.path.join(result_path,"Move")
    # create_dirs([result_path,result_aptos_path,result_sui_path,result_move_path])
    bytecode_modules_set = set()
    for root, _dirs, _files in os.walk(OPEN_SOURCE_REPO_PATH):
        if root.endswith('bytecode_modules'):
            bytecode_modules_set.add(root)
    commands = []
    for bytecode_modules_path in bytecode_modules_set:
        if "/Aptos/" in bytecode_modules_path:
            start_index = bytecode_modules_path.find("/Aptos/")
            build_index = bytecode_modules_path.find("/build/")
            end_index = bytecode_modules_path.find("/bytecode_modules")
            package = bytecode_modules_path[start_index+7:build_index] + bytecode_modules_path[build_index+6:end_index]
            result_path = os.path.join(result_aptos_path,package)
        else:
            start_index = bytecode_modules_path.find("/Sui/")
            build_index = bytecode_modules_path.find("/build/")
            end_index = bytecode_modules_path.find("/bytecode_modules")
            package = bytecode_modules_path[start_index+5:build_index] + bytecode_modules_path[build_index+6:end_index]
            result_path = os.path.join(result_sui_path,package)
        res_json_path = result_path + ".json"
        command = ["MoveScanner", "-p",bytecode_modules_path,"-o",res_json_path, "-n"]
        commands.append(command)
    # 线程数 20, 根据内核数调整
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        executor.map(run_movescanner,commands)
    logger.info("Finish. Please Check: OpenSource/result")


# 分析执行结果，获取绘图/表所需数据集
DATA_TYPE=Enum("DATA_TYPE",("PackageName_to_Time","Constant_Count","Function_Count","Detectors_to_Count","Module_Count","Project_Count"))
def open_source_analyzer(data_type:DATA_TYPE):
    result_path = os.path.join(OPEN_SOURCE_PATH,"result","json")
    if data_type == DATA_TYPE.PackageName_to_Time:
        package_name_to_time = {}
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                file_path = os.path.join(root,file)
                pattern = r"/result/json/(.*?)/(.*?)/(.*?).json"
                a,b,c= re.findall(pattern, file_path)[0]
                package_name = os.path.join(a,b,c)
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    time = json_data.get("total_time")
                    if package_name not in package_name_to_time:
                        package_name_to_time[package_name]=time
                    else:
                        logger.error("key 重复")
                        raise
        logger.info("package_name_to_time: {}".format(package_name_to_time))
        return package_name_to_time
    elif data_type == DATA_TYPE.Constant_Count:
        constant_count = 0
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                file_path = os.path.join(root,file)
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name,module in json_data.get("modules").items():
                        constant_count += module.get("constant_count")
        logger.info("constant_count: {}".format(constant_count))
        return constant_count

    elif data_type == DATA_TYPE.Function_Count:
        function_count = {
            "all":0,
            "native":0
        }
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                file_path = os.path.join(root,file)
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name,module in json_data.get("modules").items():
                        function_count["all"] += module.get("function_count").get("all")
                        function_count["native"] += module.get("function_count").get("native")
        logger.info("function_count: {}".format(function_count))
        return function_count
    elif data_type == DATA_TYPE.Detectors_to_Count:
        Detectors_to_Count = {
            "precision_loss": 0,
            "infinite_loop": 0,
            "unnecessary_type_conversion": 0,
            "overflow": 0,
            "unchecked_return": 0,
            "unused_constant": 0,
            "unnecessary_bool_judgment": 0,
            "unused_private_functions": 0,
            "recursive_function_call": 0,
            "repeated_function_call": 0
        }
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                file_path = os.path.join(root,file)
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    for _module_name,module in json_data.get("modules").items():
                        for detector_name, detector_result in module.get("detectors").items():
                            if detector_name== "repeated_function_call" and len(detector_result)!=0:
                                # logger.error(file_path)
                                logger.error(detector_result)
                            Detectors_to_Count[detector_name]+=len(detector_result)
        logger.info("function_count: {}".format(Detectors_to_Count))
        return Detectors_to_Count
    elif data_type == DATA_TYPE.Module_Count:
        Module_Count = 0
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                file_path = os.path.join(root,file)
                with open(file_path, "r") as json_file:
                    json_data = json.load(json_file)
                    Module_Count += json_data["modules_status"]["pass"]
                    Module_Count += json_data["modules_status"]["wrong"]
        logger.info("Module_Count: {}",Module_Count)
        return Module_Count
    
    elif data_type == DATA_TYPE.Project_Count:
        Project_Count = 0
        for root, _dirs, files in os.walk(result_path):
            for file in files:
                if not file.endswith('.json'):
                    continue
                Project_Count +=1

        logger.info("Project_Count: {}",Project_Count)
        return Project_Count
        
if __name__ == '__main__':
    main()