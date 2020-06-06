SELECT memo_group.id, memo_group.name, user_group.user_group_name, memo_acl.access, users.username
FROM memo_acl
JOIN memo_group on memo_acl.memo_group_id = memo_group.id
JOIN user_group_detail ON memo_acl.user_group_id = user_group_detail.user_group_id
JOIN user_group ON user_group.id = user_group_detail.user_group_id
JOIN users ON user_group_detail.user_id = users.id
WHERE memo_acl.memo_group_id = $1
AND memo_group.user_id = $2
ORDER BY user_group.id, users.id;



