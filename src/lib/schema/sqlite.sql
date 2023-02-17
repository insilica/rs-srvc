PRAGMA journal_mode=WAL;

BEGIN;

CREATE TABLE IF NOT EXISTS srvc_event (
  hash TEXT PRIMARY KEY,
  data TEXT CHECK(json_valid(data)),
  extra TEXT NOT NULL DEFAULT '{}'
    CHECK(json_valid(extra) AND json_type(extra) == 'object'),
  type TEXT NOT NULL,
  uri TEXT
) STRICT;

CREATE INDEX IF NOT EXISTS idx_srvc_event_type ON srvc_event (type);
CREATE INDEX IF NOT EXISTS idx_srvc_event_uri ON srvc_event (uri);

CREATE INDEX IF NOT EXISTS idx_srvc_event_label_answer_document
  ON srvc_event ((data->'$.document'))
  WHERE type = 'label-answer';

CREATE INDEX IF NOT EXISTS idx_srvc_event_label_answer_label
  ON srvc_event ((data->'$.label'))
  WHERE type = 'label-answer';

CREATE INDEX IF NOT EXISTS idx_srvc_event_label_answer_reviewer
  ON srvc_event ((data->'$.reviewer'))
  WHERE type = 'label-answer';

COMMIT;
