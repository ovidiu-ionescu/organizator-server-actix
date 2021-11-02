use crate::check_security_middleware::Security;
use crate::routes::{GetAllMemoTitlesQuery, GetUserQuery, MemoWrite, SearchMemoQuery, LoginQuery};
use crate::{
    errors::OrganizatorError,
    models::{GetMemo, GetWriteMemo, MemoGroup, MemoTitle, User, Login, GetFilePermissions, ExplicitPermission},
};
use deadpool_postgres::Pool;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::SystemTime;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::types::Type;

use log::debug;
use uuid::Uuid;

impl GetUserQuery {
    pub fn get_statement(&self) -> &'static str {
        include_str!("sql/get_user.sql")
    }
}

pub async fn get_user(pool: Arc<Pool>, query: GetUserQuery) -> Result<User, OrganizatorError> {
    let _stmt = query.get_statement();

    let client = pool.get().await?;
    let stmt = client.prepare_typed(&_stmt, &[Type::INT4]).await.unwrap();

    // println!("Fetching user {:#?}", query.id);
    client
        .query(&stmt, &[&query.id])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).map_err(OrganizatorError::from))
        .next()
        .unwrap()
}

pub async fn get_users(pool: Arc<Pool>) -> Result<Vec<User>, OrganizatorError> {
    let sql_stmt = include_str!("sql/get_all_users.sql");
    let client = pool.get().await?;
    let stmt = client.prepare_typed(&sql_stmt, &[]).await?;
    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).map_err(OrganizatorError::from))
        .collect()
}

impl GetAllMemoTitlesQuery {
    pub fn get_statement() -> &'static str {
        include_str!("sql/get_all_memo_titles.sql")
    }
}
pub async fn get_memo_titles(
    security: Security,
    pool: Arc<Pool>,
) -> Result<Vec<MemoTitle>, OrganizatorError> {
    let sql = GetAllMemoTitlesQuery::get_statement();

    let client = pool.get().await?;
    let stmt = client
        .prepare_typed(&sql, &[Type::VARCHAR])
        .await
        .unwrap();

    client
        .query(&stmt, &[&security.get_user_name()])
        .await?
        .iter()
        .map(|row| MemoTitle::from_row_ref(row).map_err(OrganizatorError::from))
        .collect()
}

impl SearchMemoQuery {
    pub fn get_statement(&self) -> &'static str {
        include_str!("sql/search_memo.sql")
    }
}

pub async fn search_memo(
    pool: Arc<Pool>,
    query: SearchMemoQuery,
    security: Security,
) -> Result<Vec<MemoTitle>, OrganizatorError> {
    let sql = query.get_statement();

    let client = pool.get().await?;
    let stmt = client.prepare_typed(&sql, &[Type::VARCHAR]).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &security.get_user_name(),
                &query.search.unwrap(),
            ],
        )
        .await?
        .iter()
        .map(|row| MemoTitle::from_row_ref(row).map_err(OrganizatorError::from))
        .collect()
}

pub async fn get_memo(
    pool: Arc<Pool>,
    id: i32,
    security: Security,
) -> Result<GetMemo, OrganizatorError> {
    let client = pool.get().await?;
    let stmt = client
        .prepare_typed(
            "select * from memo_read($1, $2);",
            &[Type::INT4, Type::VARCHAR],
        )
        .await
        .unwrap();

    client
        .query(
            &stmt,
            &[&id, &security.get_user_name()],
        )
        .await?
        .iter_mut()
        .map(move |row| GetMemo::from_row(row).map_err(OrganizatorError::from))
        .next()
        .unwrap()
}



fn split_memotekst<'a>(text: &'a str) -> (Option<&'a str>, Option<&'a str>) {
    let line_end = text.find(|c| c == '\n' || c == '\r');
    match line_end {
        None => (Some(text), Some("")),
        Some(i) => (Some(&text[..i]), Some(&text[i..])),
    }
}

#[cfg(test)]
mod test_memotekst {
    #[test]
    fn simple_split() {
        let body = "first\nsecond";
        let split = super::split_memotekst(&body);
        assert_eq!(Some("first"), split.0);
        assert_eq!(Some("\nsecond"), split.1);
    }

    #[test]
    fn utf_split() {
        let body = "ă\nx";
        let split = super::split_memotekst(&body);
        assert_eq!(Some("ă"), split.0);
        assert_eq!(Some("\nx"), split.1);
    }
}


fn get_millis() -> i64 {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    now.as_millis().try_into().unwrap()
}

pub async fn write_memo(
    pool: Arc<Pool>,
    memo: MemoWrite,
    security: Security,
) -> Result<GetWriteMemo, OrganizatorError> {
    let client = pool.get().await?;

    let millis = get_millis();

    let stmt = client
        .prepare_typed(
            "select * from memo_write($1, $2, $3, $4, $5, $6);",
            &[
                Type::INT4,
                Type::VARCHAR,
                Type::VARCHAR,
                Type::INT8,
                Type::INT4,
                Type::VARCHAR,
            ],
        )
        .await
        .unwrap();
    let s = memo.text.unwrap_or("".to_string());
    let b = split_memotekst(&s);

    client
        .query(
            &stmt,
            &[
                &memo.memo_id,
                &b.0,
                &b.1,
                &millis,
                &memo.group_id,
                &security.get_user_name(),
            ],
        )
        .await?
        .iter_mut()
        .map(move |row| GetWriteMemo::from_row(row).map_err(OrganizatorError::from))
        .next()
        .unwrap()
}

impl MemoGroup {
    pub fn get_all_statement() -> &'static str {
        include_str!("sql/memo_groups_for_user.sql")
    }
}

pub async fn get_memo_groups(
    pool: Arc<Pool>,
    security: Security,
) -> Result<Vec<MemoGroup>, OrganizatorError> {
    let _stmt = MemoGroup::get_all_statement();
    debug!("{}", _stmt);

    let client = pool.get().await?;
    let stmt = client
        .prepare_typed(&_stmt, &[Type::VARCHAR])
        .await
        .unwrap();

    client
        .query(&stmt, &[&security.get_user_name()])
        .await?
        .iter()
        .map(|row| MemoGroup::from_row_ref(row).map_err(OrganizatorError::from))
        .collect()
}

impl LoginQuery {
    pub fn get_statement(&self) -> &'static str {
        include_str!("sql/login.sql")
    }
}

pub async fn get_login (
    pool: &Arc<Pool>,
    login_query: &LoginQuery,
) -> Result<Login, OrganizatorError> {
    let stmt = login_query.get_statement();
    let client = pool.get().await?;
    let prepared_stmt = client
        .prepare_typed(&stmt, &[Type::VARCHAR])
        .await
        .unwrap();

    client.query(&prepared_stmt, &[&login_query.j_username])
    .await?
    .iter()
    .map(|row| Login::from_row_ref(row).map_err(OrganizatorError::from))
    .next()
    .unwrap()
}

pub async fn update_password (
    pool: &Arc<Pool>,
    username: &str,
    salt: &Vec<u8>,
    pbkdf2_hash: &Vec<u8>,
) -> Result<(), OrganizatorError> {
    let stmt = include_str!("sql/update_password.sql");
    let client = pool.get().await?;
    let prepared_stmt = client.prepare_typed(&stmt, &[Type::BYTEA, Type::BYTEA, Type::VARCHAR])
        .await
        .unwrap();
    client.execute(&prepared_stmt, &[&salt, &pbkdf2_hash, &username]).await?;
    Ok(())
}

pub async fn insert_filestore (
    pool: &Arc<Pool>,
    id: &Uuid,
    username: &str,
    filename: &str,
    memo_group_id: &Option<i32>,
) -> Result<(), OrganizatorError> {
    let stmt = include_str!("sql/insert_filestore.sql");
    let client = pool.get().await?;
    let prepared_stmt = client.prepare_typed(&stmt, &[Type::UUID, Type::VARCHAR, Type::VARCHAR, Type::INT4, Type::INT8])
        .await
        .unwrap();
    let millis = get_millis();
    client.execute(&prepared_stmt, &[&id, &username, &filename, &memo_group_id, &millis]).await?;
    Ok(())
}

pub async fn file_permissions (
    pool: &Arc<Pool>,
    id: &Uuid,
    username: &str,
    min_required: Option<i32>,
) -> Result<GetFilePermissions, OrganizatorError> {
    let stmt = include_str!("sql/get_file_security.sql");
    let client = pool.get().await?;
    let prepared_stmt = client
    .prepare_typed(&stmt, &[Type::UUID, Type::VARCHAR, Type::INT4])
    .await
    .unwrap();
    client.query(&prepared_stmt, &[&id, &username, &min_required]).await?
    .iter()
    .map(|row| GetFilePermissions::from_row(&row).map_err(OrganizatorError::from))
    .next()
    .unwrap()
}

pub async fn explicit_permissions(
    pool: &Arc<Pool>,
    username: &str,
    id: i32
) -> Result<Vec<ExplicitPermission>, OrganizatorError> {
    let stmt = include_str!("sql/explicit_permissions.sql");
    let client = pool.get().await?;
    let prepared_stmt = client
    .prepare_typed(&stmt, &[Type::INT4, Type::VARCHAR])
    .await
    .unwrap();
    client.query(&prepared_stmt, &[&id, &username]).await?
    .iter()
    .map(|row| ExplicitPermission::from_row_ref(row).map_err(OrganizatorError::from))
    .collect()
}