#[import('./example_table.csv') as table_c]

SELECT table_c.col
FROM table_c AS cost_table
WHERE cost_table.col < 10;

