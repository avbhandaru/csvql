# todo

### Blockers
 <!-- - Handle other sql to rust types in table creation. -->
 - Handle dynamic typing (using sampling or first k rows or educated guessing) and explicit type annotations.
	+ ex. `count(*)` doesn't work since it doesn't select a string, but `count(*)::TEXT` will. This is a problem since users should be able to work with numbers easily/other datatypes as well. Need to handle null case as well

### Minor
 <!-- - Make formatting/cloning of headers and widths more efficient in table.rs/just pass refs -->
 - Remove cloning and understand rust borrowing and lifetimes to optimize.
 - Make `table::Table::new` return a `Box<Table>`. This is ideal since tables can be really large and we should keep track of them in the heap?
 - Dynamic DB allocation/creation for csvql in particular
 <!-- - drop all tables upon exiting repl/runtime -->
 <!-- + Keep track of all tables we created (will be useful for `List` command as well) -->
 <!-- + And drop all of the tables we created during the repl runtime experience -->
 <!-- - GRACEFUL EXITING -->
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
 - Move path validation to `file.rs`

## Feature Tracking
 <!-- - Command Line Interface in `src/main` consider `StructOpt` and `clap::App` -->
 - Progress bar for plane sql execution
 <!-- - Type annotation processing for csv column headers -->
 - csvql table parser/substitution for execution if raw string paths are used in query
 	<!-- + Consider supporting additional `import('path/to/table.csv') as table;` -->
	+ These would be imports at the top of the sql. Could also support in repl
 - csvql repl
	<!-- + Add clear repl command! Idk how to do that... -->
	<!-- + Replace tuple enum fields with object enum fields for clarity -->
 - Support csvql repl with piped in sql query
 <!-- - Support repl file output annotations, file output as csv -->
 <!-- + Should be able to enter `select * from table;#[output]` -->
 <!-- + Annotations can look like this `#[output(=optional(pathBuf))]` -->
 + sql new tables+views should also be output-able, i.e. `create view view_name as ...;#[output='/path/to/out.csv']`
	+ Ignore most of the above, add support for `\e table/view_name`
 <!-- - Add some client side SQL validation? Or if that's not realistic, settle for propogating tokio_postgres db Error. -->
 - Testing
	+ Unit tests
	+ Integration tests
 - Add Man Page for csvql
 <!-- - Less everything! -->
 - Syntax highlighting down the line
 - Rainbow CSV/table viewing?
 <!-- - Improve Table formatting -->
 - Improve error handling for `file.rs`. Would be good if I could pass args from original error clause to the `From<OriginatingError>` to `file::Error`!
 <!-- - Figure out working directory/and relative paths in rust **DONE** -->
 - Add size in bytes column for `d+` and column description if available


## Bug Tracking
 - n/a todo

## Notes
 - n/a todo
