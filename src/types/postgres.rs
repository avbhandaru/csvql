use regex::Regex as Reg;

// Taken from tokio_postgres FromSql lib.rs

// | Rust type                         | Postgres type(s)                     |
// |-----------------------------------|--------------------------------------|
// | `bool`                            | BOOL                                 |
// | `i8`                              | "char"                               |
// | `i16`                             | SMALLINT, SMALLSERIAL                |
// | `i32`                             | INT, SERIAL                          |
// | `u32`                             | OID                                  |
// | `i64`                             | BIGINT, BIGSERIAL                    |
// | `f32`                             | REAL                                 |
// | `f64`                             | DOUBLE PRECISION                     |
// | `&str`/`String`                   | VARCHAR, CHAR(n), TEXT, CITEXT, NAME |
// | `&[u8]`/`Vec<u8>`                 | BYTEA                                |
// | `HashMap<String, Option<String>>` | HSTORE                               |
// | `SystemTime`                      | TIMESTAMP, TIMESTAMP WITH TIME ZONE  |
// | `IpAddr`                          | INET                                 |

// TODO: make all regexes space invariant/assuming Postgres SQL types allow for them to be
// TODO: allow for lowercased type annotations
lazy_static! {
  static ref BOOL_RE: Reg = Reg::new(r"^(BOOL|BOOLEAN)$").unwrap();
  static ref CHAR_RE: Reg = Reg::new(r"^char$").unwrap();
  static ref SMALL_INT_RE: Reg = Reg::new(r"^(SMALLINT|INT2)$").unwrap();
  static ref SMALL_SERIAL_RE: Reg = Reg::new(r"^SMALLSERIAL$").unwrap();
  static ref INT_RE: Reg = Reg::new(r"^(INT|INT4)$").unwrap();
  static ref SERIAL_RE: Reg = Reg::new(r"^SERIAL$").unwrap();
  static ref OID_RE: Reg = Reg::new(r"^OID$").unwrap();
  static ref BIGINT_RE: Reg = Reg::new(r"^BIGINT$").unwrap();
  static ref BIGSERIAL_RE: Reg = Reg::new(r"^BIGSERIAL$").unwrap();
  static ref REAL_RE: Reg = Reg::new(r"^(REAL|FLOAT4)$").unwrap();
  static ref DOUBLE_PRECISION_RE: Reg = Reg::new(r"^DOUBLE PRECISION$").unwrap();
  static ref NUMERIC_RE: Reg = Reg::new(r"^NUMERIC\s*\(\s*\d+\s*,\s*\d+\s*\)$").unwrap();
  static ref VARCHAR_N_RE: Reg = Reg::new(r"^VARCHAR(\(\d+\))?$").unwrap();
  static ref CHAR_N_RE: Reg = Reg::new(r"^CHAR(\(\d+\))?$").unwrap();
  static ref TEXT_RE: Reg = Reg::new(r"^TEXT$").unwrap();
  static ref CITEXT_RE: Reg = Reg::new(r"^CITEXT$").unwrap();
  static ref NAME_RE: Reg = Reg::new(r"^NAME$").unwrap();
  static ref BYTEA_RE: Reg = Reg::new(r"^BYTEA$").unwrap();
  static ref HSTORE_RE: Reg = Reg::new(r"^HSTORE$").unwrap();
  static ref TIMESTAMP_RE: Reg = Reg::new(r"^TIMESTAMP$").unwrap();
  static ref TIMESTAMP_NO_TIME_ZONE_RE: Reg = Reg::new(r"^TIMESTAMP WITHOUT TIME ZONE").unwrap();
  static ref TIMESTAMP_WITH_TIME_ZONE_RE: Reg =
    Reg::new(r"^TIMESTAMP WITH TIME ZONE [a-zA-Z]+").unwrap();
  static ref INET_RE: Reg = Reg::new(r"^INET$").unwrap();
}

// In addition, some implementations are provided for types in third party
// crates. These are disabled by default; to opt into one of these
// implementations, activate the Cargo feature corresponding to the crate's
// name prefixed by `with-`. For example, the `with-serde_json-1` feature enables
// the implementation for the `serde_json::Value` type.
//
// | Rust type                       | Postgres type(s)                    |
// |---------------------------------|-------------------------------------|
// | `chrono::NaiveDateTime`         | TIMESTAMP                           |
// | `chrono::DateTime<Utc>`         | TIMESTAMP WITH TIME ZONE            |
// | `chrono::DateTime<Local>`       | TIMESTAMP WITH TIME ZONE            |
// | `chrono::DateTime<FixedOffset>` | TIMESTAMP WITH TIME ZONE            |
// | `chrono::NaiveDate`             | DATE                                |
// | `chrono::NaiveTime`             | TIME                                |
// | `time::PrimitiveDateTime`       | TIMESTAMP                           |
// | `time::OffsetDateTime`          | TIMESTAMP WITH TIME ZONE            |
// | `time::Date`                    | DATE                                |
// | `time::Time`                    | TIME                                |
// | `eui48::MacAddress`             | MACADDR                             |
// | `geo_types::Point<f64>`         | POINT                               |
// | `geo_types::Rect<f64>`          | BOX                                 |
// | `geo_types::LineString<f64>`    | PATH                                |
// | `serde_json::Value`             | JSON, JSONB                         |
// | `uuid::Uuid`                    | UUID                                |
// | `bit_vec::BitVec`               | BIT, VARBIT                         |
// | `eui48::MacAddress`             | MACADDR                             |

lazy_static! {
  static ref DATE_RE: Reg = Reg::new(r"^(DATA|data)$").unwrap();
  static ref TIME_RE: Reg = Reg::new(r"^(TIME|time)$").unwrap();
  static ref TIME_NO_TIMEZONE_RE: Reg = Reg::new(r"^TIME WITHOUT TIME ZONE$").unwrap();
  static ref INTERVAL_RE: Reg = Reg::new(r"^INTERVAL$").unwrap();
  static ref MACADDR_RE: Reg = Reg::new(r"MACADDR^$").unwrap();
  static ref CIDR_RE: Reg = Reg::new(r"^CIDR$").unwrap();
  static ref PATH_RE: Reg = Reg::new(r"^PATH$").unwrap();
  static ref JSON_RE: Reg = Reg::new(r"^JSON$").unwrap();
  static ref JSONB_RE: Reg = Reg::new(r"^JSONB$").unwrap();
  static ref UUID_RE: Reg = Reg::new(r"^UUID$").unwrap();
  static ref BIT_RE: Reg = Reg::new(r"^BIT(\(\d+\))?$").unwrap();
  static ref VARBIT_RE: Reg = Reg::new(r"^(VARBIT|BIT VARYING)(\(\d+\))?$").unwrap();
}

// Geometry Types
lazy_static! {
  static ref POINT_RE: Reg = Reg::new(r"POINT^$").unwrap();
  static ref BOX_RE: Reg = Reg::new(r"^BOX$").unwrap();
  static ref CIRCLE_RE: Reg = Reg::new(r"^CIRCLE$").unwrap();
  static ref LINE_RE: Reg = Reg::new(r"^LINE$").unwrap();
  static ref POLYGON_RE: Reg = Reg::new(r"^POLYGON$").unwrap();
  static ref LSEG_RE: Reg = Reg::new(r"^LSEG$").unwrap();
}

// Miscellaneous Types
lazy_static! {
  static ref TSQUERY_RE: Reg = Reg::new(r"^TSQUERY$").unwrap();
  static ref TSVECTOR_RE: Reg = Reg::new(r"^TSVECTOR$").unwrap();
  static ref TXID_SNAPSHOT_RE: Reg = Reg::new(r"^TXID SNAPSHOT$").unwrap();
  static ref XML_RE: Reg = Reg::new(r"^XML$").unwrap();
  static ref MONEY_RE: Reg = Reg::new(r"^MONEY$").unwrap();
  static ref PG_LSN_RE: Reg = Reg::new(r"^PG_LSN$").unwrap();
}

// Postgres SQL Type Vector
lazy_static! {
  pub static ref TYPES: Vec<&'static Reg> = vec![
    &BOOL_RE,
    &CHAR_RE,
    &SMALL_INT_RE,
    &SMALL_SERIAL_RE,
    &INT_RE,
    &SERIAL_RE,
    &OID_RE,
    &BIGINT_RE,
    &BIGSERIAL_RE,
    &REAL_RE,
    &DOUBLE_PRECISION_RE,
    // &NUMERIC_RE,
    &VARCHAR_N_RE,
    &CHAR_N_RE,
    &TEXT_RE,
    // &CITEXT_RE,
    &NAME_RE,
    &BYTEA_RE,
    // &HSTORE_RE,
    &TIMESTAMP_RE,
    &TIMESTAMP_NO_TIME_ZONE_RE,
    &TIMESTAMP_WITH_TIME_ZONE_RE,
    &INET_RE,
    &DATE_RE,
    &TIME_RE,
    &TIME_NO_TIMEZONE_RE,
    // &INTERVAL_RE,
    // &MACADDR_RE,
    // &CIDR_RE,
    // &PATH_RE,
    // &JSON_RE,
    // &JSONB_RE,
    // &UUID_RE,
    // &BIT_RE,
    // &VARBIT_RE,
    // &POINT_RE,
    // &BOX_RE,
    // &CIRCLE_RE,
    // &LINE_RE,
    // &POLYGON_RE,
    // &LSEG_RE,
    // &TSQUERY_RE,
    // &TSVECTOR_RE,
    // &TXID_SNAPSHOT_RE,
    // &XML_RE,
    // &MONEY_RE,
    // &PG_LSN_RE
  ];
}

// Validators
pub fn is_valid_type(type_str: &str) -> bool {
  // println!("Validating this supposed type: {}", type_str);
  if type_str.is_empty() {
    false
  } else {
    for sql_type_re in TYPES.iter() {
      if (*sql_type_re).is_match(type_str) {
        return true;
      }
    }
    false
  }
}

// Resolvers
pub fn _get_type(type_str: &str) -> Option<String> {
  if type_str.is_empty() {
    None
  } else {
    for sql_type_re in TYPES.iter() {
      if sql_type_re.is_match(type_str) {
        return sql_type_re
          .captures(type_str)
          .unwrap()
          .get(0)
          .map_or(None, |m| Some(String::from(m.as_str())));
      }
    }
    None
  }
}

pub fn get_value(row: &tokio_postgres::Row, index: usize) -> String {
  // guaranteed that index is in bounds, so can unwrap
  let column = row.columns().get(index).unwrap();
  // let column_name = column.name();
  let column_type = column.type_();

  // DEBUGGING
  // println!(
  //   "Getting value for column ({}, {})",
  //   column_name,
  //   column_type.name()
  // );
  match column_type.name() {
    "BOOL" | "bool" => {
      let value: bool = row.get(index);
      format!("{}", value)
    }
    "CHAR" | "char" => {
      let value: i8 = row.get(index);
      format!("{}", value)
    }
    "INT2" | "int2" => {
      let value: i16 = row.get(index);
      format!("{}", value)
    }
    "INT4" | "int4" => {
      let value: i32 = row.get(index);
      format!("{}", value)
    }
    "INT8" | "int8" => {
      let value: i64 = row.get(index);
      format!("{}", value)
    }
    "OID" | "oid" => {
      let value: u32 = row.get(index);
      format!("{}", value)
    }
    "FLOAT4" | "float4" => {
      let value: f32 = row.get(index);
      format!("{}", value)
    }
    "FLOAT8" | "float8" => {
      let value: f64 = row.get(index);
      format!("{}", value)
    }
    "VARCHAR" | "TEXT" | "NAME" | "UNKNOWN" | "varchar" | "text" | "name" | "unknown" => {
      let value: String = row.get(index);
      format!("{}", value)
    }
    "BYTEA" | "bytea" => {
      let value: &[u8] = row.get(index);
      format!("{:?}", value)
    }
    "TIME" | "time" => {
      let value: std::time::SystemTime = row.get(index);
      format!("{:?}", value)
    }
    "TIMESTAMP" | "timestamp" => {
      let value: std::time::SystemTime = row.get(index);
      format!("{:?}", value)
    }
    "INET" | "inet" => {
      let value: std::net::IpAddr = row.get(index);
      format!("{}", value)
    }
    // TODO: support HashMap<String, Option<String>> HSTORE type
    // TODO: cover all postgresql types
    // Types can be found here: https://docs.rs/tokio-postgres/0.7.0/tokio_postgres/types/struct.Type.html
    // FromSQL common types here: https://docs.rs/tokio-postgres/0.7.0/tokio_postgres/types/trait.FromSql.html#tymethod.from_sql
    _ => {
      // could panic!
      println!("Cannot resolve type {}. Sorry!", column_type.name());
      "".to_string()
    }
  }
}
