import os
import json

# 动态获取脚本所在目录
script_dir = os.path.dirname(os.path.abspath(__file__))
directory = os.path.join(script_dir, "../result/json")
output_file_path = os.path.join(directory, "defect_summary.json")  # 输出文件路径


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
    total_count = 0
    defect_summary = {}  # 用于存储所有文件的统计结果

    for root, dirs, files in os.walk(directory):
        for filename in files:
            if filename.endswith(".json"):
                file_path = os.path.join(root, filename)
                defect_count = find_defect(file_path, defect)

                if defect_count > 0:
                    print(f"{file_path}: {defect_count}")
                    # 将结果记录到 defect_summary
                    defect_summary[file_path] = defect_count

                total_count += defect_count

    # 打印总计
    print(f"\nTotal occurrences of {defect}: {total_count}")

    # 将总计结果添加到 defect_summary
    defect_summary["total_count"] = total_count

    # 将 defect_summary 写入 JSON 文件
    with open(output_file_path, "w") as output_file:
        json.dump(defect_summary, output_file, indent=4)

    print(f"\nDefect summary has been saved to {output_file_path}")

