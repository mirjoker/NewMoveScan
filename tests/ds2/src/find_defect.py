import os
import json

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
    defect = "unnecessary_witness_cope"
    directory = "/home/jie/MoveScannerTest/OnChain/result/json"
    total_count = 0

    for root, dirs, files in os.walk(directory):
        for filename in files:
            if filename.endswith(".json"):
                file_path = os.path.join(root, filename)
                defect_count = find_defect(file_path, defect)
                
                
                total_count += defect_count

    print(f"\nTotal occurrences of {defect}: {total_count}")
