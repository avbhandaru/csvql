-- possible sql for \d, likely could improve
-- r = ordinary table, i = index, S = sequence, v = view, m = materialized view, c = composite type, t = TOAST table, f = foreign table
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
WHERE c.relkind = ANY (ARRAY['r', 'i', 'S', 'p', 'f', 'v'])
  AND n.nspname = 'public'
ORDER BY 1,2;

-- possible sql for \d table/view/etc, likely could improve
-- letting test table be 'test_table_name'
-- I want to improve the type annotating here, to CONCAT(udt_name, ' ', character_maximum_length )
-- verbose
SELECT
  a.attname AS "Column",
  pg_catalog.format_type(a.atttypid, a.atttypmod) AS "Datatype",
  CASE
    WHEN a.atthasdef THEN pg_get_expr(d.adbin, d.adrelid)
    ELSE '-'
  END AS "Default",
  NOT a.attnotnull AS "Nullable"
FROM pg_catalog.pg_attribute a
  LEFT JOIN pg_catalog.pg_attrdef d on a.attnum = d.adnum
WHERE
  a.attnum > 0
  AND NOT a.attisdropped
  AND a.attrelid = (
    SELECT c.oid
    FROM pg_catalog.pg_class c
      LEFT JOIN pg_catalog.pg_namespace n on n.oid = c.relnamespace
    WHERE c.relname = 'table' AND pg_catalog.pg_table_is_visible(c.oid)
  );

-- no type casting fix to rust
SELECT
  a.attname AS "Column",
  pg_catalog.format_type(a.atttypid, a.atttypmod) AS "Datatype",
  CASE
    WHEN a.atthasdef THEN pg_get_expr(d.adbin, d.adrelid)
    ELSE '-'
  END AS "Default",
  CASE
    WHEN NOT a.attnotnull THEN 'true'
    ELSE 'false'
  END AS "Nullable"
FROM pg_catalog.pg_attribute a
  LEFT JOIN pg_catalog.pg_attrdef d on a.attnum = d.adnum
WHERE
  a.attnum > 0
  AND NOT a.attisdropped
  AND a.attrelid = (
    SELECT c.oid
    FROM pg_catalog.pg_class c
      LEFT JOIN pg_catalog.pg_namespace n on n.oid = c.relnamespace
    WHERE c.relname = 'table' AND pg_catalog.pg_table_is_visible(c.oid)
  )

-- not verbose
SELECT
  column_name AS "Column",
  data_type AS "Datatype"
FROM information_schema.columns
WHERE (table_schema, table_name) = ('public', 'test_table_name');

-- -- below is probably more accurate
-- SELECT
--   a.attname as "Column",
--   pg_catalog.format_type(a.atttypid, a.atttypmod) as "Datatype",
--   CASE
--     WHEN a.atthasdef THEN pg_get_expr(b.adbin, b.adrelid)
--     ELSE '-'
--   END AS "Default",
--   NOT a.attnotnull AS "Nullable"
-- FROM pg_catalog.pg_attribute a
--   JOIN pg_catalog.pg_attrdef b ON a.attnum = b.adnum
--   WHERE
--     a.attnum > 0
--     AND NOT a.attisdropped
--     AND a.attrelid = (
--       SELECT c.oid
--       FROM pg_catalog.pg_class c
--         LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
--       WHERE c.relname = 'test2'
--         AND pg_catalog.pg_table_is_visible(c.oid)
--     )
