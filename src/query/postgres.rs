use crate::querier::QuerierTrait;
use crate::table;
use async_trait::async_trait;
use tokio_postgres::error::Error;
use tokio_postgres::{connect, Client, NoTls, Row};

#[derive(Debug)]
pub struct Querier {
  pub name: String,
  pub url: String,
  pub client: Client,
}

impl Querier {
  pub async fn new(querier_name: &str, database_url: &str) -> Result<Self, Error> {
    let (client, conn) = connect(&database_url, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
      if let Err(e) = conn.await {
        eprintln!("Database connection error: {}", e);
      }
    });

    Ok(Self {
      name: String::from(querier_name),
      url: String::from(database_url),
      client: client,
    })
  }
}

#[async_trait]
impl QuerierTrait for Querier {
  async fn store(
    &self,
    table_path: &str,
    table_name: &str,
    table_header: table::Header,
    // table_data: table::Rows,
  ) -> Result<(), Error> {
    self
      .client
      .query(create_table_query(&table_name, &table_header).as_str(), &[])
      .await?;
    self
      .client
      .query(
        copy_into_query(&table_path, &table_name, &table_header).as_str(),
        // insert_into_query(&table_name, &table_header, table_data).as_str(),
        &[],
      )
      .await?;
    Ok(())
  }

  async fn query(&self, query_statement: &str) -> Result<Option<table::Table>, Error> {
    let rows: Vec<Row> = self.client.query(query_statement, &[]).await?;
    if rows.len() == 0 {
      return Ok(None);
    }

    let header_response: table::Header = rows
      .get(0)
      .unwrap()
      .columns()
      .iter()
      .map(|column_info| {
        (
          String::from(column_info.name()),
          String::from(column_info.type_().name()),
        )
      })
      .collect::<Vec<_>>();

    let rows_response: table::Rows = rows
      .into_iter()
      .map(|row| {
        let mut row_vector = Vec::new();
        for (col_index, _) in row.columns().iter().enumerate() {
          row_vector.push(row.get(col_index));
        }
        row_vector
      })
      .collect::<Vec<_>>();

    Ok(Some(table::Table::new(header_response, rows_response)))
  }

  async fn load(
    &self,
    table_name: &str,
    num_rows: Option<usize>,
  ) -> Result<Option<table::Table>, Error> {
    let query;
    if num_rows == None {
      query = format!("SELECT * FROM {}", table_name);
    } else {
      query = format!("SELECT * FROM {} LIMIT {}", table_name, num_rows.unwrap())
    }
    self.query(query.as_str()).await
  }

  async fn drop(&self, table_name: &str) -> Result<(), Error> {
    match self
      .query(format!("DROP TABLE {}", table_name).as_str())
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e),
    }
  }

  async fn list(&self) -> Result<Option<table::Table>, Error> {
    self.query(list_tables_query().as_str()).await
  }

  async fn info(&self, table_name: &str, is_verbose: bool) -> Result<Option<table::Table>, Error> {
    self
      .query(get_table_info_query(table_name, is_verbose).as_str())
      .await
  }
}

// HELPERS
fn create_table_query(table_name: &str, table_header: &table::Header) -> String {
  let schema = table_header
    .into_iter()
    .map(|(col_name, col_type)| format!("{} {}", col_name, col_type))
    .collect::<Vec<_>>()
    .join(",");
  let query = format!("CREATE TABLE {} ({})", table_name, schema);
  // println!("{}", query);
  query
}

/*
  csvql_db=# COPY first_table (source,text,created_at,retweet_count,favorite_count,is_retweet,id_str)
  csvql_db-# FROM '/Users/akhil/csvql/data/test.csv'
  csvql_db-# DELIMITER ','
  csvql_db-# CSV HEADER;
*/
fn copy_into_query(table_path: &str, name: &str, header: &table::Header) -> String {
  let header = header
    .into_iter()
    .map(|(col_name, _)| col_name.as_str())
    .collect::<Vec<_>>()
    .join(",");
  let query = format!(
    "COPY {} ({}) FROM '{}' DELIMITER ',' CSV HEADER",
    name, header, table_path
  );
  // println!("query: {}", query);
  query
}

fn list_tables_query() -> String {
  let query = format!(
    "
    SELECT
      n.nspname AS \"Schema\",
      c.relname AS \"Table\",
      CASE
        WHEN c.relkind = 'r' THEN 'table'
        WHEN c.relkind = 'i' THEN 'index'
        WHEN c.relkind = 'S' THEN 'sequence'
        WHEN c.relkind = 'v' THEN 'view'
        WHEN c.relkind = 'f' THEN 'foreign table'
      END AS \"Type\",
      a.rolname AS \"Owner\"
    FROM pg_catalog.pg_class c
      LEFT JOIN pg_catalog.pg_namespace n
      ON n.oid = c.relnamespace
      LEFT JOIN pg_catalog.pg_authid a
      ON c.relowner = a.oid
    WHERE c.relkind = ANY (ARRAY['r', 'i', 'S', 'p', 'f', 'v'])
      AND n.nspname = 'public'
    ORDER BY 1,2;
    "
  );
  query
}

fn get_table_info_query(table_name: &str, is_verbose: bool) -> String {
  if is_verbose {
    let query = format!(
      "
      SELECT
        a.attname AS \"Column\",
        pg_catalog.format_type(a.atttypid, a.atttypmod) AS \"Datatype\",
        CASE
          WHEN a.atthasdef THEN pg_get_expr(d.adbin, d.adrelid)
          ELSE '-'
        END AS \"Default\",
        CASE
          WHEN NOT a.attnotnull THEN 'true'
          ELSE 'false'
        END AS \"Nullable\"
      FROM pg_catalog.pg_attribute a
        LEFT JOIN pg_catalog.pg_attrdef d on a.attnum = d.adnum
      WHERE
        a.attnum > 0
        AND NOT a.attisdropped
        AND a.attrelid = (
          SELECT c.oid
          FROM pg_catalog.pg_class c
            LEFT JOIN pg_catalog.pg_namespace n on n.oid = c.relnamespace
          WHERE c.relname = '{}' AND pg_catalog.pg_table_is_visible(c.oid)
        )
      ",
      table_name
    );
    query
  } else {
    let query = format!(
      "
      SELECT
        column_name AS \"Column\",
        data_type AS \"Datatype\"
      FROM information_schema.columns
      WHERE (table_schema, table_name) = ('public', '{}');
      ",
      table_name
    );
    query
  }
}

// TODO --> consider removing, since we have copy
//
// MAKE SURE THIS IS WORKING
//
// Run with example in repl:
// > \i /Users/akhil/csvql/data/test.csv first
// > select * from first_table;
fn _insert_into_query(
  table_name: &str,
  table_header: &table::Header,
  table_data: table::Rows,
) -> String {
  let header = table_header
    .into_iter()
    .map(|(col_name, _)| col_name.as_str())
    .collect::<Vec<_>>()
    .join(",");
  let values = table_data
    .into_iter()
    .map(|row| {
      let row_values = row
        .into_iter()
        .map(|elt| {
          // TODO perform some string validation and escapting:
          // psql string with escapes is
          // E'meow I like cats \\n meow he isn\\'t the coolest'
          format!("'{}'", elt)
        })
        .collect::<Vec<_>>()
        .join(",");
      format!("({})", row_values)
    })
    .collect::<Vec<_>>()
    .join(",");
  let query = format!("INSERT INTO {} ({}) VALUES {}", table_name, header, values);
  query
}
