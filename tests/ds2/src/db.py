import os,json
from loguru import logger
CURRENT_PATH = os.path.dirname(os.path.abspath(__file__))
ONCHAIN_DB_PATH = os.path.join(os.path.dirname(CURRENT_PATH),"res","db")
APTOS_DB_PATH = os.path.join(ONCHAIN_DB_PATH,"aptos_db.json")
SUI_DB_PATH = os.path.join(ONCHAIN_DB_PATH,"sui_db.json")

class DB:
    def __init__(self,type:str):
        if type.lower() == "sui":
            logger.info("sui_db 数据最后更新于 2024/01/31")
            db_path = SUI_DB_PATH
        elif type.lower() == "aptos":
            logger.info("aptos_db 数据最后更新于 2024/01/31")
            db_path = APTOS_DB_PATH
        else:
            raise Exception("type must be sui or aptos")
        if not os.path.exists(db_path):
            raise Exception("db file not found: {}".format(db_path))
        with open(db_path,"r") as f:
            db_data = json.load(f)
            self.column_names = db_data["meta"]["column_names"]
            self.row_count = db_data["meta"]["row_count"]
            self.content = db_data["db_content"]
            
    def get_row(self,row_index:int):
        return self.content[row_index]