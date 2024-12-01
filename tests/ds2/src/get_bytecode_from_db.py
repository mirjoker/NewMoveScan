from db import DB
from loguru import logger
import tqdm,time,os
CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ONCHAIN_BYTECODE_PATH = os.path.join(os.path.dirname(CURRENT_PATH),"res","bytecode")
SUI_BYTECODE_PATH = os.path.join(ONCHAIN_BYTECODE_PATH,"Sui")
APTOS_BYTECODE_PATH = os.path.join(ONCHAIN_BYTECODE_PATH,"Aptos")

if not os.path.exists(SUI_BYTECODE_PATH):
    os.makedirs(SUI_BYTECODE_PATH)
    logger.info(f"create {SUI_BYTECODE_PATH} success")
if not os.path.exists(APTOS_BYTECODE_PATH):
    os.makedirs(APTOS_BYTECODE_PATH)
    logger.info(f"create {APTOS_BYTECODE_PATH} success")

earliest_time = time.time()
latest_time = 0

def get_sui_bytecode_from_db():
    global earliest_time
    global latest_time
    sui_db = DB("sui")
    status = {
        "success"  : 0,
        "skip":0
    }
    with tqdm.tqdm(total=sui_db.row_count,desc="Sui") as pbar:
        for row in sui_db.content:
            pbar.update(1)
            publish_time = int(row[5])
            if publish_time < earliest_time:
                earliest_time = publish_time
            if publish_time > latest_time:
                latest_time = publish_time
            bytecode_file_name = f"{row[0]}_{row[1]}_{row[2]}.mv"
            bytecode_file_path = os.path.join(SUI_BYTECODE_PATH, bytecode_file_name)
            if os.path.exists(bytecode_file_path):
                # logger.warning("file already exist, skip. {}".format(bytecode_file_path))
                status["skip"]+=1
            else:
                with open(bytecode_file_path, "wb") as f:
                    f.write(bytes.fromhex(row[3]))
                    status["success"]+=1
                    # logger.info("write file success. {}".format(bytecode_file_path))
    logger.success(f"sui_db 中共 {sui_db.row_count} 条数据，成功写入 {status['success']} 条, 已存在跳过写入 {status['skip']} 条. 查看: {SUI_BYTECODE_PATH}")
    
def get_aptos_bytecode_from_db():
    global earliest_time
    global latest_time
    aptos_db = DB("aptos")
    status = {
        "success"  : 0,
        "skip":0
    }
    with tqdm.tqdm(total=aptos_db.row_count,desc="Aptos") as pbar:
        for row in aptos_db.content:
            pbar.update(1)
            publish_time = int(row[5])
            if publish_time < earliest_time and publish_time != 0:
                earliest_time = publish_time
            if publish_time > latest_time:
                latest_time = publish_time
            bytecode_file_name = f"{row[0]}_{row[1]}_{row[2]}.mv"
            bytecode_file_path = os.path.join(APTOS_BYTECODE_PATH, bytecode_file_name)
            if os.path.exists(bytecode_file_path):
                # logger.warning("file already exist, skip. {}".format(bytecode_file_path))
                status["skip"]+=1
            else:
                with open(bytecode_file_path, "wb") as f:
                    f.write(bytes.fromhex(row[3]))
                    status["success"]+=1
                    # logger.info("write file success. {}".format(bytecode_file_path))
    logger.success(f"aptos_db 中共 {aptos_db.row_count} 条数据，成功写入 {status['success']} 条, 已存在跳过写入 {status['skip']} 条. 查看: {APTOS_BYTECODE_PATH}")

def human_time(timestamp):
    return time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(timestamp))
    
def main():
    logger.info("Step1: 从 aptos_db 获取 aptos 链上字节码")
    get_aptos_bytecode_from_db()
    logger.info("Step2: 从 sui_db 获取 sui 链上字节码")
    get_sui_bytecode_from_db()
    logger.info(f"earliest: {human_time(earliest_time)} latest: {human_time(latest_time)}")
    
if __name__ == '__main__':
    main()
