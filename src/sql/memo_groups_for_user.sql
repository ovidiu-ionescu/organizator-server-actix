SELECT memo_group.id, memo_group.name, memo_group.user_id
FROM memo_group
JOIN users ON user_id = users.id
WHERE users.username = $1;

