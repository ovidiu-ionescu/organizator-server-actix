# Run this as psql
# CREATE EXTENSION unaccent;
# ALTER FUNCTION unaccent(text) IMMUTABLE;

CREATE INDEX memo_fulltext_idx ON memo USING gin(to_ts_vector(unaccent (title || memotext)));