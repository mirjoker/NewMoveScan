# OpenSource Analysis

Sources of open-source projects (excluding those that failed to compile or are empty):

https://github.com/MystenLabs/awesome-move
https://github.com/aptos-foundation/ecosystem-projects

The project list and data features can be found in res/features/.

# Usage

1. Extract the compiled open-source projects:
```shell
cd NewMoveScan/tests/ds1/res/repo
unzip Sui.zip 
unzip Aptos.zip
```

2. Analyze using MoveScanner:

```shell
cd ../src
python open_source_analyzer.py
```

3. View the results in `/ds1/result`

# Test
```shell
cd src
# Must use bash; ./run.sh cannot be used
bash run.sh <path of bytecode_modules> 
```

Test results will be output to `src/output`
