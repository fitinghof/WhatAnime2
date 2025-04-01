-- Add migration script here
CREATE OR REPLACE FUNCTION array_unique(arr1 anyarray, arr2 anyarray)
RETURNS anyarray AS $$
  SELECT array_agg(DISTINCT element)
  FROM unnest(COALESCE(arr1, '{}') || COALESCE(arr2, '{}')) AS t(element);
$$ LANGUAGE sql;
