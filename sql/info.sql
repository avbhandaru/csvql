-- possible sql for \d, likely could improve
SELECT
  n.nspname AS "Schema",
  c.relname AS "Table",
  CASE
    WHEN c.relkind = 'r' THEN 'table'
    WHEN c.relkind = 'i' THEN 'index'
    WHEN c.relkind = 'S' THEN 'sequence'
    WHEN c.relkind = 'v' THEN 'view'
    WHEN c.relkind = 'f' THEN 'foreign table'
  END AS "Type",
  a.rolname AS "Owner"
FROM pg_catalog.pg_class c
  LEFT JOIN pg_catalog.pg_namespace n
  ON n.oid = c.relnamespace
  LEFT JOIN pg_catalog.pg_authid a
  ON c.relowner = a.oid
-- r = ordinary table, i = index, S = sequence, v = view, m = materialized view, c = composite type, t = TOAST table, f = foreign table
WHERE c.relkind = ANY (ARRAY['r', 'i', 'S', 'p', 'f', 'v'])
  AND n.nspname = 'public'
ORDER BY 1,2;

-- possible sql for \d table/view/etc, likely could improve
-- letting test table be 'test_table_name'
-- I want to improve the type annotating here, to CONCAT(udt_name, ' ', character_maximum_length )
SELECT
  column_name AS "Column",
  CASE
    WHEN data_type = 'character varying' THEN CONCAT(data_type, ' (', character_maximum_length, ')')
    ELSE data_type
  END AS "Datatype",
  CASE
    WHEN column_default IS NULL THEN 'n/a'
    ELSE column_default
  END AS "Default",
  is_nullable AS "Nullable"
FROM information_schema.columns
WHERE (table_schema, table_name) = ('public', 'test_table_name');

-- below is probably more accurate
-- SELECT
--         a.attname as "Column",
--         pg_catalog.format_type(a.atttypid, a.atttypmod) as "Datatype"
--     FROM
--         pg_catalog.pg_attribute a
--     WHERE
--         a.attnum > 0
--         AND NOT a.attisdropped
--         AND a.attrelid = (
--             SELECT c.oid
--             FROM pg_catalog.pg_class c
--                 LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
--             WHERE c.relname = 'test_table_name'
--                 AND pg_catalog.pg_table_is_visible(c.oid)
--         );
