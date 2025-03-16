# NewMoveScan

NewMoveScan is a bytecode-based static analysis tool for Move smart contracts. This tool is written in Rust and designed to analyze and detect issues in Move smart contract bytecode.

## QuickStart

This repository contains the pre-built NewMoveScan executable. You do not need to compile it yourself.

**Step 1.** Download the latest release from the Releases section.

This release contains a pre-built executable for Linux only. It will not work on other operating systems such as Windows or macOS.

Download the corresponding executable file.

**Step 2.** Running NewMoveScan

Once the executable is downloaded and set up, you can start using NewMoveScan directly from the command line.

```shell
$ ./NewMoveScan -h  #ubuntu20.04
```

## Usage

```
$ NewMoveScan -h
A static analysis tool based on bytecode for move smart contracts.

Usage: NewMoveScan [OPTIONS] --path <PATH> [COMMAND]

Commands:
  printer
  detector
  help      Print this message or the help of the given subcommand(s)

Options:
  -p, --path <PATH>        Path to input dir/file
  -o, --output <OUTPUT>    Path to output file [default: result.json]
  -n, --none               Print nothing on terminal
  -j, --json               Print result as json on terminal
  -i, --ir-type <IR_TYPE>  IR type [possible values: sb, cm, cfg, du, fs, cg]
  -h, --help               Print help
  -V, --version            Print version
```
NewMoveScan supports the analysis of an entire project or an individual module within a project. For analyzing a project/module, you should provide the path to the compiled project/module.

```shell
$ NewMoveScan -p <bytecode_dir_path>
$ NewMoveScan -p <bytecode_file_path>
```
## Examples

To analyze the project in ds1:

```shell
$ ./NewMoveScan -p tests/ds1/Sui/poly-stc-contracts/build/Poly-STC-Bridge/bytecode_modules
```

```shell
pass: 31     wrong: 3      time: 9868 us

no: 0
module_name: LockProxy
unnecessary_access_control: [ init_event ] 
unnecessary_emit: [ emit_fee_lock_event ] 

no: 1
module_name: StarcoinVerifier
unnecessary_access_control: [ create ]

no: 2
module_name: TokenMock
unnecessary_access_control: [ register_token ] 
```

To analyze the module in ds2:

```shell
$ ./NewMoveScan -p test/ds2/Aptos/1_0x1_account.mv
```

```shell
pass: 0      wrong: 1      time: 2210 us

no: 0
module_name: account
unnecessary_access_control: [ initialize ]
```
