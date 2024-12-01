"""
@Desc: 获取 aptos 链上数据，存储于 aptos_db.json 中
@Author: HappyTsing
"""
import requests
from loguru import logger
from time import sleep,strptime,mktime
import os
import json
CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ONCHAIN_DB_PATH = os.path.join(os.path.dirname(CURRENT_PATH),"res","db")
APTOS_DB_PATH = os.path.join(ONCHAIN_DB_PATH,"aptos_db.json")
LOG_PATH = os.path.join(CURRENT_PATH, "create_aptos_db.log")
logger.add(LOG_PATH, format='{time} | {level}    | {message}')

BASE_URL = "https://api.chainbase.online/v1/dw/query"

CHAINBASE_API_KEY = "2SNh1gpw9v08zwRhHXm4bOJNIP5"

HEADER = {
    "x-api-key": CHAINBASE_API_KEY
}

if not os.path.exists(ONCHAIN_DB_PATH):
    os.makedirs(ONCHAIN_DB_PATH)
    logger.info("db dir not exist, create. {}".format(ONCHAIN_DB_PATH))
    
if not os.path.exists(APTOS_DB_PATH):
    logger.info("aptos db not exist, init. {}".format(APTOS_DB_PATH))
    aptos_db = {
        "meta":{
            "row_count" : 0,
            "column_names" : ["id","address",",module_name","bytecode","transaction_block_height","timestamp"]
        },
        "db_content":[]
    }
else:
    logger.info("aptos db exist, read. {}".format(APTOS_DB_PATH))
    with open(APTOS_DB_PATH,"r") as f:
        aptos_db = json.load(f)
        
def db_update(id,address, moudle_name, bytecode,transaction_block_height):
    # 判断是否重复更新(因为是按顺序索引的，因此 id 相同时，其内容也一定相同)
    # 此处 int(id) < (int(aptos_db["meta"]["row_count"]) + 1) 表示小于 row_count + 1 的 id 都已经被存储到数据库中，因此跳过
    if int(id) < (int(aptos_db["meta"]["row_count"]) + 1):
        logger.info("db update failed. id {} already exist. skip.".format(id))
        return
    else:
        bytecode = bytecode.lstrip("\\x")
        timestamp = get_block_timestamp(transaction_block_height)
        line = [id,address,moudle_name,bytecode,transaction_block_height,timestamp]
        aptos_db["meta"]["row_count"] += 1
        aptos_db["db_content"].append(line)
        logger.info("db update success. {}".format(line))
    # 每更新 100 条数据覆盖写一次
    if aptos_db["meta"]["row_count"] % 100 == 0:
        db_store()
        
def db_store():
    with open(APTOS_DB_PATH,"w") as f:        
        json.dump(aptos_db,f)

def covert_time(time_str):
    time_array = strptime(time_str, "%Y-%m-%d %H:%M:%S")
    timestamp = int(mktime(time_array))
    return str(timestamp)
    
def get_block_timestamp(block_height):
    payload = {"query": "select timestamp from aptos.user_transactions where block_height ={}".format(block_height)}
    response = requests.post(BASE_URL, json=payload, headers=HEADER)
    timestamp = 0 # block_height = 0 时，查不到
    if response.status_code == 200:
        data = response.json().get("data")
        lines = data.get("result")
        for line in lines:
            timestamp = covert_time(line["timestamp"])
    else:
        logger.error("api request error: {}".format(response.status_code))
        raise
    sleep(0.65) # 免费版 20 req/10sec，加点延时
    return timestamp

def create_aptos_db():
    base_payload = {"query": "select row_number() over(order by transaction_block_height) as id,address,name,transaction_block_height,bytecode from aptos.move_modules order by transaction_block_height"}
    response = requests.post(BASE_URL, json=base_payload, headers=HEADER)
    if response.status_code == 200:
        data = response.json().get("data")
        lines = data.get("result")
        next_page = data.get("next_page")
        for line in lines:
            id = line.get("id") # 从 1 开始编号
            address = line.get("address")
            module_name = line.get("name")
            transaction_block_height = line.get("transaction_block_height")
            bytecode = line.get("bytecode")
            db_update(id,address, module_name, bytecode,transaction_block_height)
        # 免费版 20 req/10sec，加点延时
        sleep(2)
        while(next_page):
            logger.info("重新获取 task_id，因为长时间等待上一个 task 已经过期")
            response = requests.post(BASE_URL, json=base_payload, headers=HEADER)
            if response.status_code == 200:
                data = response.json().get("data")
                task_id = data.get("task_id")
            sleep(2)
            logger.info("正在处理第 {} 页（1000 条/页）...".format(next_page))
            next_payload = {"task_id":task_id,"page":next_page}
            response = requests.post(BASE_URL, json=next_payload, headers=HEADER)
            if response.status_code == 200:
                data = response.json().get("data")
                lines = data.get("result")
                next_page = data.get("next_page")
                for line in lines:
                    id = line.get("id") # 从 1 开始编号
                    address = line.get("address")
                    module_name = line.get("name")
                    transaction_block_height = line.get("transaction_block_height")
                    bytecode = line.get("bytecode")
                    if bytecode:
                        db_update(id,address, module_name, bytecode,transaction_block_height)
                    else:
                        logger.info("bytecode is None, Skip. address: {}",address)
                sleep(2)
            else:
                logger.error("api request error: {}".format(response.status_code))
                raise
    else:
        logger.error("api request error: {}".format(response.status_code))
        raise

def main():
    restart_flag = True
    while(restart_flag):
        try:
            restart_flag = False # 假设此时运行能成功
            create_aptos_db()
            logger.success("create aptos db success!")
        except Exception as e:
            restart_flag = True  # 遇到异常，需要重启
            logger.info("catch exception, retry...")
    db_store()

if __name__ == '__main__':
    main()
