DROP FUNCTION memo_write;
/*

Permissions:
1) Requester has to exist
2) New memo can't be empt
3) Memo if specified, has to exist
4) If title, memotext and group_id are the same, don't do the update
5) Owner can only change group_id to another one he owns
Owner of memo can change title, memotext. 

Another user can if allowed, only change memotext

*/


CREATE OR REPLACE FUNCTION memo_write(
  p_memo_id       memo.id%TYPE, 
  p_memo_title    memo.title%TYPE, 
  p_memo_memotext memo.memotext%TYPE, 
  p_memo_group_id memo.group_id%TYPE,
  p_username      users.username%TYPE,
  p_savetime      memo.savetime%TYPE,
  
  OUT o_id             memo.id%TYPE,
  OUT o_requester_id   users.id%TYPE,
  OUT o_requester_name users.username%TYPE,
  OUT o_dbg int
  )
  AS $$
  DECLARE
    v_empty_content      boolean;
    v_old_title     p_memo_title%TYPE;
    v_old_memotext       p_memo_memotext%TYPE;
    v_old_memo_group_id  p_memo_group_id%TYPE;
    v_memo_user_id       memo.user_id%TYPE;
    v_memo_group_user_id memo_group.user_id%TYPE;
    v_access  memo_acl.access%TYPE;
  BEGIN
    BEGIN
    -- 1) check the requester user exists
    SELECT users.id INTO STRICT o_requester_id FROM users WHERE users.username = p_username;
    EXCEPTION 
      WHEN NO_DATA_FOUND THEN
        RAISE EXCEPTION 'user % not found', p_username USING ERRCODE = '28000'; -- invalid_authorization_specification
      WHEN TOO_MANY_ROWS THEN
        RAISE EXCEPTION 'fetched more than one user for %', p_username USING ERRCODE = '28000'; -- invalid_authorization_specification
    END;

    BEGIN
    -- fetch the owner of the memo group
      IF p_memo_group_id IS NOT NULL THEN
        SELECT user_id INTO STRICT v_memo_group_user_id FROM memo_group WHERE id = p_memo_group_id;
      END IF;
    EXCEPTION 
      WHEN NO_DATA_FOUND THEN
        RAISE EXCEPTION 'memo group % not found', p_memo_group_id USING ERRCODE = '02000'; -- no_data
    END;
      
    -- is the content of the new memo empty?
    v_empty_content := LENGTH(COALESCE(p_memo_title, '')) + LENGTH(COALESCE(p_memo_memotext, '')) = 0;

    -- are we creating a new memo?
    IF p_memo_id IS NULL THEN
      IF v_empty_content THEN
        -- 2) new memo can't be empty
        RAISE EXCEPTION 'You can not create an empty new memo' USING ERRCODE = '02000'; -- no_data
      END IF;

      -- create the new memo
      NULL;
    ELSE
      --update an existing memo
      
      -- fetch the old memo
      BEGIN
        SELECT memo.title, memo.memotext, memo.group_id, memo.user_id
	  INTO STRICT v_old_title, v_old_memotext, v_old_memo_group_id, v_memo_user_id
	  FROM memo
         WHERE memo.id = p_memo_id;
      EXCEPTION
      WHEN NO_DATA_FOUND THEN
        -- 3) Memo if specified, has to exist
        RAISE EXCEPTION 'memo % not found', p_memo_id USING ERRCODE = '02000'; -- no_data
      END;
      IF (p_memo_title IS NOT DISTINCT FROM v_old_title) 
	  AND (p_memo_memotext IS NOT DISTINCT FROM v_old_memotext) 
	  AND (p_memo_group_id IS NOT DISTINCT FROM v_old_memo_group_id)
	 THEN
	  -- 4) the previous memo is exactly the same, there's no need to save anything
	  RAISE NOTICE 'Memo values for % did not change, not saving', p_memo_id;
        RETURN;
      END IF;
      IF p_memo_group_id IS NOT NULL AND v_memo_group_user_id <> v_memo_user_id THEN
        -- 5) Owner can only change group_id to another one he owns
        RAISE EXCEPTION 'New memogroup belongs to user %, not user % who owns memo %',
          v_memo_group_user_id, v_memo_user_id, p_memo_id
          USING ERRCODE = '2F002'; -- modifying_sql_data_not_permitted
      END IF;

      IF o_requester_id = v_memo_user_id THEN
        -- owner of the memo has full rights on the memo
        IF v_empty_content THEN
          -- delete the memo
          NULL;
        ELSE
          -- modify the memo
          NULL;
        END IF;
      ELSE
        -- requester is not owner, can only modify memotext
        IF p_memo_title <> v_old_title THEN
          RAISE EXCEPTION 'user % (%) is not allowed to modify title for memo % because memo is owned by %',
            o_requester_id, p_username, p_memo_id, v_memo_user_id
            USING ERRCODE = '2F002'; -- modifying_sql_data_not_permitted
        END IF;
        IF p_memo_group_id <> v_old_memo_group_id THEN
          RAISE EXCEPTION 'user % (%) is not allowed to modify group id for memo % because memo is owned by %',
            o_requester_id, p_username, p_memo_id, v_memo_user_id
            USING ERRCODE = '2F002'; -- modifying_sql_data_not_permitted
        END IF;
	-- get permission for user
	SELECT memo_group_user_access(p_memo_group_id, o_requester_id) INTO v_access;

        -- update the memo
        NULL;
      END IF;
      
    END IF;
    -- are we deleting an existing memo?
    -- does the group id belong to the memo owner?
    -- is the memo any different?
    -- does it belong to this user?
    -- write the old version to history
  END; $$
LANGUAGE 'plpgsql';