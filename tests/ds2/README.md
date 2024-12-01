# OnChain Analysis

Sui Data Rang: 2024-04-13 01:00:00 - 2024-07-23 20:10:52
Aptos Data Range: 2024-3-14 09:37:36 - 2023-07-24 16:34:27

**1. Extract the Prepared Database**

> Data collected on: 2024-07-23.
If you choose to use this data, you can skip steps 2 and 3.

```shell
cd NewMoveScan/tests/ds2/res
unzip db.zip
```

After extraction, you will obtain two database files: `aptos_db.json` and `sui_db.json`.
The format of the data in each file is as follows:

```json
{
  "meta":{
    "row_count" : 0,
    "column_names" : ["id","address",",module_name","bytecode","transaction_block_height","timestamp"]
  },
  "db_content":[
    [...],[...] // Each list represents a row, with its contents corresponding to the column_names.
  ]
}
```

>  In the sui_db, the `transaction_block_height` field will always be None, which is normal since this information is currently unavailable.

**2. Run Scripts to Fetch the Latest Data and Build db.json**
>  If you want to retrieve the latest data and build `aptos_db.json` and `sui_db.json` yourself, run this step. Otherwise, skip it.
```shell
cd NewMoveScan/tests/ds2/src
python create_aptos_db.py  # Approx. 6 hours (uses API calls to fetch timestamps, subject to rate limits)
python create_sui_db.py    # Approx. 30 minutes
```


**3. Generate Input Files for MoveScanner Based on db.json**

```shell
cd NewMoveScan/tests/ds2/src
python get_bytecode_from_db.py
```

After running this, two folders, `Sui` and `Aptos`, will be created in `MoveScanner/OnChain/res/bytecode`. These folders will contain the executable files required for MoveScanner testing.

**4. Test Using MoveScanner**

```shell
cd NewMoveScan/tests/ds2/src
python onchain_analyzer.py
```
The test results will be stored in:`NewMoveScan/tests/ds2/result/json/Aptos` and `NewMoveScan/tests/ds2/result/json/Sui` .


