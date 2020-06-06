drop table filestore;
CREATE TABLE filestore (
  id UUID PRIMARY KEY, 
  user_id INTEGER NOT NULL, 
  filename VARCHAR(255), 
  memo_group_id INTEGER,
  uploaded_on BIGINT,
  CONSTRAINT filestore_user_id_fkey FOREIGN KEY (user_id)
    REFERENCES users (id) MATCH SIMPLE
    ON DELETE CASCADE,
  CONSTRAINT filestore_memo_group_id_fkey FOREIGN KEY (memo_group_id)
    REFERENCES memo_group(id) MATCH SIMPLE
 );