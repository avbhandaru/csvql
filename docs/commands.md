# Commands
**SUBJECT TO CHANGE** - please refer to `todo.md`

## Description
**`this`** documents the command line interface for the **`csvql`** tool.

Running **`csvql`** with the `-h` or `--help` will equivalently describe the following commands.

## Example Usage
Key:

 - `/path/to/table_i.csv` will simply be referred to as `ti`
 - `/path/to/out.csv` or `.txt` will simply be referred to as `out`
 - `path/to/query_i.csv` will simply be referred to as `qi`.
 - `list[ti | qi]` is equivalent to `t1 t2 ... tn` and `q1 q2 ... qm`, respectively.

> `csvql`

> `csvql list[ti]`

> `csvql list[ti] query.sql`

> `csvql list[ti] query.sql out.csv`

> `query.csv | csvql list[ti]`

> `query.csv | csvql list[ti] out.csv`

## **`csvql [-args]`**
Below are possible args and usage specifications.

### `--imports t1 t2 ... tn`
Opens up a repl with tables `t1 t2 ... tn` loaded and named `table_i` (i.e. their file name without extension and path prefix).

### `--imports t1 t2 ... tn --exports out`
Opens up a repl with tables `t1 t2 ... tn` loaded and named `table_i` (i.e. their file name without extension and path prefix).

Directs all default outputs to the output file `out`. csv data is appended to the outfile (a new line `\n` will precede all appended outputs).

### `--imports list[ti] --queries list[qi]`
Loads tables `t1 t2 ... tn` named `table_i` (i.e. their file name without extension and path prefix).

Executes all queries `q1 q2 ... qm` and outputs them to standard out. Output will be styled (not csv).

### `--imports list[ti] --queries list[qi] --exports out`
Loads tables `t1 t2 ... tn` named `table_i` (i.e. their file name without extension and path prefix).

Executes all queries `q1 q2 ... qm` and outputs them to the output file `out`. csv data is appended to the outfile (a new line `\n` will precede all appended outputs).

### `--queries q1 q2 ... qm`
Resolves all table imports (header or inline) in query files.

Executes all queries `q1 q2 ... qm` and outputs them to standard out. Output will be styled (not csv).

### `--queries q1 q2 ... qm --exports out`
Resolves all table imports (header or inline) in query files.

Executes all queries `q1 q2 ... qm` and outputs them to the output file `out`. csv data is appended to the outfile (a new line `\n` will precede all appended outputs).

### `--json`
If this flag is present then all outputs will be in `JSON` format rather than `csv`, or styled (in repl), the default.

`JSON` will be appended to `.csv` files if `.csv` files are given as the exports output file. No file extension validation will occur.
