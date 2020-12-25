# todo
 - Make formatting/cloning of headers and widths more efficient in table.rs/just pass refs
 - Remove cloning and understand rust borrowing and lifetimes to optimize.
 - Make `table::Table::new` return a `Box<Table>`. This is ideal since tables can be really large and we should keep track of them in the heap?
 - Dynamic DB allocation/creation for csvql in particular
 - drop all tables upon exiting repl/runtime
	+ Keep track of all tables we created (will be useful for `List` command as well)
	+ And drop all of the tables we created during the repl runtime experience
 - GRACEFUL EXITING
 - HANDLE GIANT TABLE buffers? Learn to print in chunks rather than all at once? With the `Less` command this causes a "too many arguments" error within the terminal (i.e. max string size is limited by ~1/4 stack size)
	+ Reproduce using:
	```shell
	> \i /Users/akhil/csvql/data/test.csv first
	> \i /Users/akhil/csvql/data/test.csv second
	```
	```sql
	> SELECT first.text, first.retweet_count AS count
	  FROM first
		JOIN second
		ON first.source = second.source;
	```
	+ The above results in query response that is too large
	+ This however works:
	```sql
	> SELECT first.text, first.retweet_count AS count
	  FROM first
		JOIN second
		ON first.id_str = second.id_str;
	```
	+ This is because `id_str` is a unique id
 - GET Postgres to Rust TYPE CONVERSIONS WORKING WHEN QUERYING!
 - Pad table entries for pretty display

## Misc Tracking
 - Pseudo-finalize `docs/commands.md`
 - Grab `DATABASE_URL` for postgres and link with diesel code
 - Figure out how the hell Rust Macros across different files works...
 - Move Help and Usage and Other in code docs/strings to a static file/or a const string file? Possibly look into `https://doc.rust-lang.org/std/process/struct.Command.html` terminal commands to use `HEAD` or `LESS` on static files and get the nice vim bindings like in man pages and psql!

## Feature Tracking
 - Command Line Interface in `src/main` consider `StructOpt` and `clap::App`
 - Progress bar for plane sql execution
 - Type annotation processing for csv column headers
 - csvql table parser/substitution for execution if raw string paths are used in query
 	+ Consider supporting additional `import('path/to/table.csv') as table;`
	+ These would be imports at the top of the sql. Could also support in repl
 - csvql repl
	+ Add clear repl command! Idk how to do that...
	+ Replace tuple enum fields with object enum fields for clarity
 - Support csvql repl with piped in sql query
 - Support repl file output annotations, file output as csv
	+ Should be able to enter `select * from table;#[output]`
	+ Annotations can look like this `#[output(=optional(pathBuf))]`
	+ sql new tables+views should also be output-able, i.e. `create view view_name as ...;#[output='/path/to/out.csv']`
	+ Ignore most of the above, add support for `\e table/view_name`
 - Add some client side SQL validation? Or if that's not realistic, settle for propogating tokio_postgres db Error.
 - Testing
	+ Unit tests
	+ Integration tests
 - Add Man Page for csvql
 - Less everything!
 - Syntax highlighting down the line
 - Rainbow CSV/table viewing?
 - Improve Table formatting
 - Improve error handling for `file.rs`. Would be good if I could pass args from original error clause to the `From<OriginatingError>` to `file::Error`!
 - Figure out working directory/and relative paths in rust

## Bug Tracking
 - n/a todo

## Notes
```
## stdin means we can do the following with pipes:
# cat query.sql | csvql
# cat query.sql | csvql table.csv
# cat query.sql | csvql --imports ti.csv
# cat query.sql | csvql --imports ti.csv > out.csv
## and so on

csvql stdin
csvql stdin > out.csv
csvql --imports ti.csv stdin
csvql --imports ti.csv stdin > out.csv

csvql repl --imports ti.csv --exports out.csv
csvql repl --imports ti.csv
csvql repl

csvql exec --queries qj.sql --imports ti.csv  --exports oj.csv
csvql exec --queries qj.sql --imports ti.csv  --exports out.csv
csvql exec --queries qj.sql --exports oj.csv
csvql exec --queries qj.sql --exports out.csv
csvql exec --queries qj.sql --exports oj.csv --json
```

Some updated REPL help and usage strings:
```rust
let help =
	"
	Terminology:
		PATH              - an absolute or relative path to a csv (imports) or json file (exports can be csv or json)

	General:
		\\q               - Quit repl
		\\?               - Show help on backslash commands (this page)
		\\h               - Show usage examples for (csvql)
		\\print bool      - If bool is false then no resulting query rows will be printed to repl, vice versa

	Import:
		\\i path          - Imports a csv table into the database given a PATH
		\\i path name     - Imports a csv table into the database given a PATH and aliases the table with given name
		\\import          - Equivalent long form of above, same usages

	Export:
		\\e path          - Exports last query result into csv file given a PATH, equivalent to (\\e 1 path)
		\\e n path        - Exports n(th) last query (1 being most recent, max 5 query history size) into csv file
		\\e j path        - Equivalent to (e path), but exports as json
		\\e j n path      - Equivalent to (e n path), but exports as json
		\\export          - Equivalent long form of above, same usages

	Informational:
		\\d[+]            - list tables, views and sequences, with additional information if (+) is used
		\\d[+] name       - describe table, view, sequence, or index, with additional information if (+) is used
	";

let usage =
	"
	Querying the Database:
		Regular sql code followed by a semi-colon.
		Following are some examples.

		> SELECT table.a, table.b
			FROM table
			JOIN other_table ON table.a = other_table.a;
			--------------------------------------------
				a    | b | c
			--------+---+----
			ribbit | 1 | a
			woof   | 2 | an
			meow   | 7 | ask
							.
							.
							.

		> CREATE TABLE other_other_table (
				a SERIAL,
				b VARCHAR(256),
			);
			--------------------------------
			Successfully created table with name 'other_other_table'

		Any resulting rows from a select statement will be printed to the repl.
		This stdout feature can be suppresed using a repl backslash command (\\print).

	Importing Tables:
		Use the (\\?) command to find out the exact syntax for import statements.
		Following are some examples.

		> \\i '~/home/csv_tables/other_table.csv'
			---------------------------------------
			Successfully imported table from '~/home/csv_tables/other_table.csv' as 'other_table'

		> \\i '/Users/home/csv_tables/asdfghjkl.csv' random_letters_table
			---------------------------------------------------------------
			Successfully imported table from '~/home/csv_tables/asdfghjkl.csv' as 'random_letters_table'

		The above will import the csv table from the given file.
		By default all column types will be set to VARCHAR with max size of 1 GB.
		Planning on supporting sql type annotations in csv column header.

	Exporting Tables:
			Use the (\\?) command to find out the exact syntax for export statements.

			> \\print false
			> SELECT * FROM random_letters_table;
			> \\e '~/home/csv_tables/exported_table.csv'
				------------------------------------------
				Successfully exported last query:
					'SELECT * FROM random_letters_table'
				into csv file '~/home/csv_tables/exported_table.csv'

			This will export the resulting rows of the last query into the provided csv file.
			If the file does not already exist, it will be created for you.

			> \\e 5 '~/home/csv_tables/5th_exported_table.csv'

			In this case the 5th most recent (5th to last) query result will be exported.

			> \\e j 2 '/Users/home/json_tables/2nd_exported_table.json'

			In this case, the penultimate (2nd to last) query result will be formatted as json
			and exported to the provided json file.

	Getting Information:
		Use the (\\?) command to find out the exact syntax on informational statements.
		These work in the same way the psql command (\\d[S+] [name]) works.

		> \\d
			---
			Schema |         Name         |   Type   | Owner
			--------+----------------------+----------+-------
			public | table                | table    | user
			public | table_id_seq         | sequence | user
			public | other_table          | table    | user
			public | other_other_table    | table    | user
			public | other_table_seq      | sequence | user

		> \\d other_other_table
			---------------------
				Column |          Type          | Collation | Nullable |             Default
			--------+------------------------+-----------+----------+---------------------------------
				a      | integer                |           | not null | nextval('test_a_seq'::regclass)
				b      | character varying(256) |           |          |
				c      | character varying(128) |           |          |

	";
```
