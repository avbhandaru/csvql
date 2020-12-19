SELECT
  a.table_catalog,
  a.table_schema,
  a.table_name,
  b.column_name
FROM (
  SELECT *
  FROM information_schema.tables
  WHERE table_schema = 'public'
) AS a
JOIN (
  SELECT table_name, column_name
  FROM information_schema.columns
) AS b
ON a.table_name = b.table_name;