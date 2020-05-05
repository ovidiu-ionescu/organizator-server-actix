DROP FUNCTION memo_group_user_access;
CREATE OR REPLACE FUNCTION memo_group_user_access (
  p_memo_group_id memo_group.id%TYPE,
  p_user_id users.id%TYPE
  )
  RETURNS memo_acl.access%TYPE AS $$
DECLARE
  v_access memo_acl.access%TYPE;
BEGIN
  SELECT MAX(memo_acl.access) INTO v_access
    FROM user_group,
         user_group_detail,
         memo_acl
   WHERE user_group.id = user_group_detail.user_group_id
     AND user_group.id = memo_acl.user_group_id
     AND user_group_detail.user_id = p_user_id -- requesting user is in this group
     -- and user_group.user_id <> o_requester_id -- not owner of the group
     AND memo_acl.memo_group_id = p_memo_group_id;

  IF v_access IS NULL THEN
    RAISE EXCEPTION 'User % does not have permissions on memo group %', p_user_id, p_memo_group_id
      USING ERRCODE = '2F003'; -- prohibited_sql_statement_attempted
  END IF;
  
  RETURN v_access;
END;
$$ LANGUAGE 'plpgsql';