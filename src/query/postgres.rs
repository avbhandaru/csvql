use crate::querier::{Header, Querier, QuerierTrait, QueryResponse, Rows};
use async_trait::async_trait;
use tokio;
use tokio_postgres::{connect, Error, NoTls, Row};

#[async_trait]
impl QuerierTrait for Querier {
  async fn new(querier_name: &str, database_url: &str) -> Result<Self, Error> {
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

  async fn store(
    &self,
    table_name: &str,
    table_header: Header,
    table_data: Rows,
  ) -> Result<(), Error> {
    self
      .client
      .query(create_table_query(&table_name, table_header).as_str(), &[])
      .await?;
    self
      .client
      .query(insert_into_query(&table_name, table_data).as_str(), &[])
      .await?;
    Ok(())
  }

  async fn query(&self, query_statement: &str) -> Result<QueryResponse, Error> {
    let rows: Vec<Row> = self.client.query(query_statement, &[]).await?;

    let header_response: Header = rows
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

    let rows_response: Rows = rows
      .into_iter()
      .map(|row| {
        let mut row_vector = Vec::new();
        for (col_index, column) in row.columns().iter().enumerate() {
          row_vector.push(row.get(col_index));
        }
        row_vector
      })
      .collect::<Vec<_>>();

    Ok(QueryResponse {
      header: header_response,
      rows: rows_response,
    })
  }

  async fn load(&self, table_name: &str) -> Result<QueryResponse, Error> {
    self
      .query(format!("SELECT * FROM {}", table_name).as_str())
      .await
  }
}

// HELPERS
fn create_table_query(table_name: &str, table_header: Header) -> String {
  let schema = table_header
    .into_iter()
    .map(|(col_name, col_type)| format!("{} {}", col_name, col_type))
    .collect::<Vec<_>>()
    .join(",");
  format!("CREATE TABLE {} ({})", table_name, schema)
}

fn insert_into_query(table_name: &str, table_data: Rows) -> String {
  let values = table_data
    .into_iter()
    .map(|row| {
      let row_values = row.join(",");
      format!("({})", row_values)
    })
    .collect::<Vec<_>>()
    .join(",");
  format!("INSERT INTO {} VALUES ({})", table_name, values)
}
