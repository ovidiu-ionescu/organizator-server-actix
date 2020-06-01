DROP FUNCTION file_user_access;
CREATE OR REPLACE FUNCTION file_user_access(
  IN    i_id              filestore.id%TYPE,
  OUT   o_requester_id    users.id%TYPE,
  INOUT io_requester_name users.username%TYPE,
  OUT   o_user_id         users.id%TYPE,
  OUT   o_username       users.username%TYPE,
  OUT   o_memo_group_id   memo_group.id%TYPE,
  IN    i_min_required    memo_acl.access%TYPE,
  OUT   o_access          int
  )
  AS $$
  DECLARE
  BEGIN
    BEGIN
      -- 1) check the requester user exists
      SELECT users.id INTO STRICT o_requester_id FROM users WHERE users.username = io_requester_name;
      EXCEPTION 
        WHEN NO_DATA_FOUND THEN
          RAISE EXCEPTION 'user % not found', io_requester_name USING ERRCODE = '28000'; -- invalid_authorization_specification
        WHEN TOO_MANY_ROWS THEN
          RAISE EXCEPTION 'fetched more than one user for %', io_requester_name USING ERRCODE = '28000'; -- invalid_authorization_specification
    END;

    BEGIN
      SELECT filestore.user_id, filestore.memo_group_id, users.username INTO o_user_id, o_memo_group_id, o_username
      FROM filestore
      JOIN users ON filestore.user_id = users.id
      WHERE filestore.id = i_id;
      EXCEPTION 
        WHEN NO_DATA_FOUND THEN
          RAISE EXCEPTION 'file % not found', i_id USING ERRCODE = '02000'; -- no_data
    END;
    IF o_user_id = o_requester_id THEN
      o_access := 10;
      RETURN;
    END IF;

    -- if we are here the requester is not the file owner
    IF o_memo_group__id IS NULL THEN
      IF i_min_requred IS NOT NULL THEN
     TODO       RAISE EXCEPTION 'User % does not have permissions on file %', p_user_id, p_memo_group_id
      USING ERRCODE = '2F003'; -- prohibited_sql_statement_attempted

      ELSE
        o_access := 0;
        RETURN;
      END IF
      
    END IF;
    -- check permission for user
    SELECT memo_group_user_access(o_memo_group_id, o_requester_id, i_min_required) INTO o_access;
  END; $$
LANGUAGE 'plpgsql';