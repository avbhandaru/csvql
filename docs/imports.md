# Imports

## Description
**`this`** documents the ways in which one can import `.csv` files and use them as tables in their executed sql code or repl expressions.

Normally, tables are created and populated using the `CREATE TABLE _` and `INSERT INTO _ VALUES` statements. In **csvql**, one can instead import tables and provide them aliases using (approximately) rust attribute syntax. These import statements go at the top and provide a way for **`csvql`** to know where to look for the data and how one plans to refer to it in their following query.

Alternatively, **`csvql`** also supports _inline_ `.csv` file paths as opposed to using table name aliases.

And alternatively again, we can choose to call **`csvql`** with the `.csv` tables already loaded using the file splat command arguments:
```sh
~/ $ csvql path/to/table_1.csv path/to/table_2.csv ... table_n.csv query.sql
```
Noting the above will load in the `.csv` files and expect that the table names used in the `sql` code are the exact file names of the tables used in the command. The command will then immediately execute the query.

Leaving out the `query.sql` file will simply open up the repl with the provided tables loaded and ready to query from.

## Examples
Suppose we have a file tree as such:
```sh
.
├── data
│   ├── table_1.csv
│   ├── table_1.filtered.csv
│   ├── table_2.csv
│   ├── table_2.filtered.csv
│   ├── fake_data.csv
│   └── filter.js
├── package-lock.json
├── package.json
└── src
    ├── index.js
    ├── server.js
    .
    .
    .
```

And we want to select data from table_1 and table_2. Then in order to query these tables and load them in using **`csvql`** we can do the following:

```sql
#[import(path/to/table_1.csv) as my_data]
#[import(path/to/table_2.csv)]

SELECT my_data.a, my_data.b, my_data.c, table_2.a
FROM my_data
JOIN table_2
ON my_data.d = table_2.d;
```

Table name aliasing also works:

```sql
#[import(path/to/table_1.csv) as my_data]
#[import(path/to/table_2.csv)]

SELECT t1.a, t1.b, t1.c, t2.a
FROM my_data AS t1
JOIN table_2 AS t2
ON t1.d = t2.d;
```

Slightly uglier, we can import the tables inline:

```sql
SELECT table_1.a, t1.b, t1.c, t2.a
FROM #[import(path/to/table_1.csv)] AS t1
JOIN #[import(path/to/table_2.csv) as t2]
ON t1.d = t2.d;
```

Note `table_1` and `t1` both work in the above query, but only `t2` would be a valid table name for `table_2.csv` as we perform an internal aliasing with the import.

Similarly, within the repl we can import new `.csv` tables on the fly (so long as we know the absolute or relative path to the new table):

```
> #[import(path/to/table.csv) as t]
  loaded in table "path/to/table.csv" as t

> SELECT * FROM t WHERE t.id < 1000;
  |id | a | b |  c  |
  |-----------------|
  | 1 | 2 | 4 | 0.5 | 
  | 2 | 1 | 4 | 0.33|
  | 7 | 2 | 3 | 0.1 |
  | 11| 2 | 4 | 9.5 |
  .
  .
  .
```

## Specifications
`#[import(path/to/table.csv) = table_alias]`
