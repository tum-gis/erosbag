CREATE TABLE topics(
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  serialization_format TEXT NOT NULL,
  offered_qos_profiles TEXT NOT NULL
);
CREATE TABLE messages(
  id INTEGER PRIMARY KEY NOT NULL,
  topic_id INTEGER NOT NULL,
  timestamp BIGINT NOT NULL, -- needed to be changes to derive i64
  data BLOB NOT NULL
);
CREATE INDEX timestamp_idx ON messages (timestamp ASC);
