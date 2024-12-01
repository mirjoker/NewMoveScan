"""
@Desc: 获取 sui 链上数据，存储于 sui_db.json
@Author: HappyTsing
"""
import requests
from loguru import logger
from time import sleep
import time
import os
import base64
from tqdm import tqdm
import json
CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ONCHAIN_DB_PATH = os.path.join(os.path.dirname(CURRENT_PATH),"res","db")
SUI_DB_PATH = os.path.join(ONCHAIN_DB_PATH,"sui_db.json")
LOG_PATH = os.path.join(CURRENT_PATH, "create_sui_db.log")
logger.add(LOG_PATH, format='{time} | {level}    | {message}')

SUI_BASE_URL = "https://explorer-rpc.mainnet.sui.io/"
SUI_HEADER = {
    "Content-Type": "application/json"
}
SUISCAN_BASE_URL = "https://suiscan.xyz/api/sui-backend/mainnet/api/"
SUISCAN_HEADER = {
    "Content-Type": "application/json"
}

if not os.path.exists(ONCHAIN_DB_PATH):
    os.makedirs(ONCHAIN_DB_PATH)
    logger.info("db dir not exist, create. {}".format(ONCHAIN_DB_PATH))
    
if not os.path.exists(SUI_DB_PATH):
    logger.info("sui db not exist, init. {}".format(SUI_DB_PATH))
    sui_db = {
        "meta":{
            "row_count" : 0,
            "column_names" : ["id","address",",module_name","bytecode","transaction_block_height","timestamp"]
        },
        "db_content":[]
    }

def db_update(address, moudle_name, bytecode_base64,timestamp,transaction_block_height=None): # sui 的 transaction_block_height 拿不到
    id = sui_db["meta"]["row_count"]
    binary_bytecode = base64.b64decode(bytecode_base64)  # 解码
    str_bytecode = binary_bytecode.hex()
    line = [id,address,moudle_name,str_bytecode,transaction_block_height,timestamp]
    sui_db["meta"]["row_count"] += 1
    sui_db["db_content"].append(line)
    logger.info("db update success. {}".format(line))
      

def db_store():
    with open(SUI_DB_PATH,"w") as f:        
        json.dump(sui_db,f)

# 将毫秒时间戳转换为秒级
def covert_time(timestamp_ms):
    timestamp_s = int(timestamp_ms) // 1000
    return str(timestamp_s)
# 从 suiscan 获取 package_id
def get_package_id_list_by_suiscan():
    package_id_map = {}
    page_size = 2000
    url = "{}packages?page=0&sortBy=TRANSACTIONS&orderBy=DESC&size={}".format(
        SUISCAN_BASE_URL, page_size)
    response = requests.get(url, headers=SUISCAN_HEADER)
    if response.status_code == 200:
        data = response.json()
        is_last = data.get("last")
        content = data.get("content")
        page_number = data.get("number")
        # logger.error(data) # 只有时间戳
        next_number = page_number + 1
        for package in content:
            package_id_map[package.get("packageId")] = package.get("timestamp")
        while (not bool(is_last)):
            logger.info("正在处理第 {} 页（{}条/页）...".format(next_number, page_size))
            url = "{}packages?page={}&sortBy=TRANSACTIONS&orderBy=DESC&size={}".format(SUISCAN_BASE_URL, next_number, page_size)
            response = requests.get(url, headers=SUISCAN_HEADER)
            if response.status_code == 200:
                data = response.json()
                is_last = data.get("last")
                content = data.get("content")
                page_number = data.get("number")
                # logger.error(data) # 只有时间戳
                next_number = page_number + 1
                for package in content:
                    package_id_map[package.get("packageId")] = package.get("timestamp")
            else:
                logger.error("api request error: {}, status code: {}".format(
                    response.text, response.status_code))
                raise
    else:
        logger.error("api request error: {}, status code: {}".format(response.text, response.status_code))
    logger.info("共从 suiscan 数据库共获得 {} 个不重复 package id".format(len(package_id_map)))
    return package_id_map

# 批量获取 package 的数据，其中包含字节码，该字节码为 base64 的格式，需要解码
def packages_handler(package_id_map):
    package_id_list = list(package_id_map.keys())
    logger.error(package_id_list)
    # sui api 一次最多处理 50 个，因此分成多次，每次传 30 个
    subset_len = 30 
    for package_id_sublist in tqdm([package_id_list[i:i + subset_len] for i in range(0, len(package_id_list), subset_len)]):
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sui_multiGetObjects",
            "params": [
                # package_id_list 最长为 50
                package_id_sublist,
                {
                    "showType": True,
                    "showOwner": False,
                    "showContent": False,
                    "showBcs": True,
                }
            ]
        }
        response = requests.post(SUI_BASE_URL, json=payload, headers=SUI_HEADER)
        if response.status_code == 200:
            package_info_list = response.json().get("result")
            for package_info in package_info_list:
                object_id   = package_info["data"]["objectId"]
                bcs_module_map = package_info["data"]["bcs"]["moduleMap"]
                timestamp = covert_time(package_id_map[object_id])
                for module_name, bytecode_base64 in bcs_module_map.items():
                    db_update(object_id, module_name, bytecode_base64,timestamp)
        else:
            logger.error("api request error: {}, status code: {}".format(
                response.text, response.status_code))
            raise

def main():
    logger.info("Step1: 请求 suiscan 获取 package_id 和 timestamp")
    package_id_map = get_package_id_list_by_suiscan()
    logger.info("Step2: 调用 sui api 获取 bytecode")
    packages_handler(package_id_map)
    logger.info("Step3: 持久化存储 sui_db")
    db_store()
    
if __name__ == '__main__':
    main()
