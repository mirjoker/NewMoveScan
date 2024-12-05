import os
import json

import os

# 动态获取脚本所在目录
script_dir = os.path.dirname(os.path.abspath(__file__))
directory = os.path.join(script_dir, "../result/json")


def find_defect(file_path, defect):
    count = 0
    with open(file_path, "r") as file:
        data = json.load(file)

    modules = data.get("modules", {})
    for module in modules.values():
        defects = module["detectors"].get(defect, [])
        count += len(defects)
    return count

if __name__ == "__main__":
    defect = "unnecessary_access_control"
    total_count = 0

    for root, dirs, files in os.walk(directory):
        for filename in files:
            if filename.endswith(".json"):
                file_path = os.path.join(root, filename)
                defect_count = find_defect(file_path, defect)
                
                if defect_count > 0:
                    print(f"{file_path}: {defect_count}")
                
                total_count += defect_count

    print(f"\nTotal occurrences of {defect}: {total_count}")
