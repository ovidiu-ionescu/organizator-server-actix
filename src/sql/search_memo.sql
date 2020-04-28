select id, title, user_id, savetime 
  from memo 
 where to_tsvector(unaccent(title || memotext)) @@ to_tsquery(unaccent($2))
   -- either own memos or shared by others
   and (
     user_id in (
       select id 
         from users 
        where users.username = $1
     )
    or 
    id in (
      select memo.id
        from memo
       where memo.group_id in (
        select memo_acl.memo_group_id
          from user_group,
               user_group_detail,
               memo_acl,
               users
         where user_group.id = user_group_detail.user_group_id
           and user_group.id = memo_acl.user_group_id
           and user_group_detail.user_id = users.id
           and user_group.user_id <> users.id
           and users.username = $1
       )      
    ) --end look for memos shared with this user

   ) -- end and condition


union all

select 0, $1, users.id, 0
  from users
 where users.username = $1
;