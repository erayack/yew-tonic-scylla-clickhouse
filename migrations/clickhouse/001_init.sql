CREATE TABLE IF NOT EXISTS events_analytics (
  id UUID,
  name String,
  created_at DateTime64(3, 'UTC')
)
ENGINE = MergeTree
ORDER BY created_at;
