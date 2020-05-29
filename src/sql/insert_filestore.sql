INSERT INTO
  filestore(id, user_id, filename, memo_group_id, uploaded_on)
SELECT $1, users.id, $3, memo_group.id, $5
FROM users LEFT JOIN memo_group ON users.id = memo_group.user_id AND memo_group.id = $4
WHERE users.username = $2
;