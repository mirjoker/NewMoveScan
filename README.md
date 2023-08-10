# MoveScanner

MoveScanner is a bytecode based move static analysis tool written in rust.

## QuickStart

1. Run `build.sh`:

   ```shell
   cd MoveScanner
   ./build
   ```

2. Please add the following to your shell configuration file(e.g., `~/.bashrc`, `~/.zshrc`):

   ```shell
   vim ~/.bashrc # Choose your own shell configuration file
   export MOVESCANNER_ROOT="$HOME/.MoveScanner"
   export PATH="$MOVESCANNER_ROOT/bin:$PATH"
   ```

3. If you want to update MoveScanner:

   ```
   git pull
   ./buildls
   ```

4. Start a new terminal session, enjoy!

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
MoveScanner -p "./testdata/examples_mv/aptos"

# -f <bytecode_file>
MoveScanner -p "./testdata/examples_mv/aptos/overflow.mv"
```

The result is output to `result.json` by default, you can customize the output file name and path by running `-o`：

```shell
# filename
MoveScanner -p "./testdata/examples_mv/aptos" -o my_result.json

# path and filename
MoveScanner -p "./testdata/examples_mv/aptos" -o /my/path/my_result.json
```

if you want to print result as json on termianl, use `-j`

### Printer

The printer can output some intermediate representations:

- `sb`: Stackless Bytecode
- `cm`: Compile Module
- `cfg`: Control Flow Graph
- `du`: Tempindex def and use
- `fs`: Function Signatures
- `cg`: Function Call Graph

```shell
MoveScanner -p "./testdata/examples_mv/aptos" -i sb printer
```
## Detector Define

| id  | Detector                  | Define                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| --- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | InfiniteLoop              | Infinite loop refers to a situation in a contract where a loop construct exists, but the loop body fails to satisfy the termination condition, resulting in an infinite repetition of the loop.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2   | Overflow                  | The term "overflow" used here specifically refers to the situation where an overflow occurs due to the SHL instruction.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 3   | PrecisionLoss             | Precision loss refers to a situation in which a significant discrepancy arises between the calculated result and the actual result due to the occurrence of operations involving multiplication, division, and square root in an order where division or square root is performed before multiplication.In the case of consecutive multiplication and division operations, the result obtained by performing multiplication first and then division is closer to the true result compared to performing division first and then multiplication. This defect also occurs in consecutive square root multiplication operations, where performing the square root operation before the multiplication operation results in greater precision loss. |
| 4   | UncheckedReturn           | Unchecked return refers to the situation that when a function call occurs in the move contract, the called function has return values, but the caller does not receive the return value or only receives part of the return value.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 5   | UnnecessaryBoolJudgment   | Unnecessary bool judgment defect is another form of meaningless code construction. It refers to the situation where a boolean variable is compared for equality or inequality with a boolean constant in conditional statements, which is equivalent to directly using the boolean variable as the conditional expression.                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 6   | UnnecessaryTypeConversion | Unnecessary type conversion refers to the situation where a typecast operation is performed on an integer variable, even though the original and target types are the same. This results in an unnecessary "cast" instruction in the bytecode, leading to gas wastage.                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 7   | UnusedConstant            | Unused constants refer to constants that are defined within a module but remain unused in the module's code. It is important to note that the module mentioned here does not include test code because the bytecode generated by test code is not included in the bytecode deployed on the blockchain. The main consequence of the Unused Constants defect is the increase in gas costs during module deployment, leading to gas wastage.                                                                                                                                                                                                                                                                                                       |
| 8   | UnusedPrivateFunctions    | Similar to unused constants, unused private functions refer to the situation where private functions are declared and defined within a module but remain unused within the same module. This renders the declared and defined private functions inaccessible and essentially dead code. This not only leads to gas wastage similar to Unused Constants but also suggests the possibility of the programmer mistakenly setting the function's visibility.                                                                                                                                                                                                                                                                                        |

## Contribute

Contributions welcome! Click [here](https://movebit1.yuque.com/xcnnsm/cf_records/gdxeo9s6r5miv4pm) to make suggestions to make MoveScanner better!
