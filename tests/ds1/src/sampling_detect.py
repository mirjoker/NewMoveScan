import os
import random

def load_output_file(file_path, sample_size=20):
    """
    读取 output.txt 文件并随机抽取样本
    """
    with open(file_path, 'r') as file:
        lines = file.readlines()

    # 解析每一行，提取地址、模块名和函数名
    functions = []
    current_address = ""
    for line in lines:
        line = line.strip()
        if line.startswith("/home"):
            current_address = line
        elif "Module" in line and "Function" in line:
            module_function = line.replace("Module: ", "").replace("Function: ", "")
            functions.append(f"{current_address}, {module_function}")

    # 计算总数
    total_population = len(functions)

    if total_population < sample_size:
        sample_size = total_population  # 如果函数总数少于样本数，全部取出

    # 随机抽取样本
    sample_functions = random.sample(functions, sample_size)

    return sample_functions

# 主函数，加载文件并打印抽取的样本
def main():
    # 文件路径
    output_file_path = '/home/jie/MoveScannerTest/OpenSource/src/output.txt'
    
    if os.path.exists(output_file_path):
        sample_functions = load_output_file(output_file_path, sample_size=20)
        print(f"抽取的 {len(sample_functions)} 个样本:")
        for func in sample_functions:
            print(func)
    else:
        print(f"文件 {output_file_path} 不存在。")

if __name__ == "__main__":
    main()
