# MoveScanner

MoveScanner is a bytecode based move static analysis tool written in rust.

## QuickStart

**Step 1.** Run `build.sh`:

```shell
cd MoveScanner
./build.sh
```
Use 32-bit addresses as the default value(move32), support for detecting projects compiled by aptos and sui, if you want to detect move projects, please use `./build move20` to compile.

**Step 2.** Configure Shell (Option)

`build.sh` will automatically configure MoveScanner for your default shell. 

If you wish to use MoveScanner on another shell, you should add the following to shell configuration file.

```shell
export MOVESCANNER_ROOT="$HOME/.MoveScanner"
export PATH="$MOVESCANNER_ROOT/bin:$PATH"
```

Otherwise, you can skip `Step 2`.

**Step 3.** If you want to update MoveScanner:

```
git pull
./build.sh
```

**Step 4.** Start a new terminal session, enjoy!

Datasets ds1 and ds2 are located in the test folder.


## Usage

```
$ MoveScanner -h
A static analysis tool based on bytecode for move smart contracts.

Usage: MoveScanner [OPTIONS] --path <PATH> [COMMAND]

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

### Detector

**detector is default executor**, so you can omit it when using it.

```shell
# -f <bytecode_dir>
# Tips: Normally you should input 'build/.../bytecode_modules'
MoveScanner -p "./res/examples_mv/aptos"

# -f <bytecode_file>
MoveScanner -p "./res/examples_mv/aptos/overflow.mv"
```

The result is output to `result.json` by default, you can customize the output file name and path by running `-o`：

```shell
# filename
MoveScanner -p "./res/examples_mv/aptos" -o my_result.json

# path and filename
MoveScanner -p "./res/examples_mv/aptos" -o /my/path/my_result.json
```

if you want to print result as json on termianl, use `-j`, if you don't want to output results on the command line, use `-n`.

### Printer

The printer can output some intermediate representations:

- `sb`: Stackless Bytecode
- `cm`: Compile Module
- `cfg`: Control Flow Graph
- `du`: Tempindex def and use
- `fs`: Function Signatures
- `cg`: Function Call Graph

```shell
MoveScanner -p "./res/examples_mv/aptos" -i sb printer
```

## Detector Define

| id  | Detector                  | Define                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| --- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Unchecked Zero            | U.Zero refers to the absence of an assertion check confirming that the liquidity provider's token is greater than zero in liquidity pool functions. This vulnerability results in users not receiving liquidity provider tokens and incurring losses of the tokens they injected into the liquidity pool.                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2   | Unnecessary Emit          | U.Emit refers to a function that solely emits a transaction, yet is a public function, meaning anyone can call it to emit transactions. Malicious calls may cause subsequent logical errors.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 3   | Unnecessary Access Control| U.Control refers to the lack of necessary verification within a function to determine whether the caller is the actual owner of an asset. This omission allows attackers to transfer users' assets, potentially resulting in financial losses for the asset owners.                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 4   | Unnecessary Witness Copy  | U.Copy in Move programming refers to the improper assignment of copy capability to a witness struct, resulting in operations intended to execute only once potentially being performed multiple times. The witness design pattern is used to control single-use operations. For instance, this applies to minting rights, where unauthorized users could gain repeated minting rights, allowing them to issue new tokens at will, causing unlimited token inflation and undermining transaction system security.                                                                                                                                                                                                                                |

## Contribute

Contributions welcome! Click [here](https://movebit1.yuque.com/xcnnsm/cf_records/gdxeo9s6r5miv4pm) to make suggestions to make MoveScanner better!
