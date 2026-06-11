CREATE KEYSPACE IF NOT EXISTS app
WITH replication = {
  'class': 'SimpleStrategy',
  'replication_factor': 1
};

CREATE TABLE IF NOT EXISTS app.events (
  id uuid PRIMARY KEY,
  name text,
  payload text,
  created_at timestamp
);
