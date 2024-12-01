import os
import json

def find_defect(file_path, defect):
    """
    查找每个文件中指定缺陷的详细位置，包括模块和函数名
    """
    results = []
    
    with open(file_path, "r") as file:
        data = json.load(file)

    # 遍历模块，获取检测结果
    for module_name, module_data in data.get("modules", {}).items():
        defects = module_data["detectors"].get(defect, [])
        for defect_instance in defects:
            # 记录每个漏洞的模块名和具体函数名
            results.append(f"Module: {module_name}, Function: {defect_instance}")
    
    return results

if __name__ == "__main__":
    defect = "unnecessary_access_control"
    directory = "/home/jie/MoveScannerTest/OpenSource/result/json"
    total_count = 0

    # 遍历指定目录下的所有json文件
    for root, dirs, files in os.walk(directory):
        for filename in files:
            if filename.endswith(".json"):
                file_path = os.path.join(root, filename)
                # 查找每个文件中所有对应缺陷的详细位置
                defect_details = find_defect(file_path, defect)
                
                if defect_details:
                    print(f"{file_path}:")
                    for detail in defect_details:
                        print(f"  {detail}")
                
                # 更新总计数
                total_count += len(defect_details)

    print(f"\nTotal occurrences of {defect}: {total_count}")
