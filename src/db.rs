use crate::check_security_middleware::Security;
use crate::routes::{GetAllMemoTitlesQuery, GetUserQuery, MemoWrite, SearchMemoQuery, LoginQuery};
use crate::{
    errors::OrganizatorError,
    models::{GetMemo, GetWriteMemo, MemoGroup, MemoTitle, User, Login},
};
use deadpool_postgres::Pool;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::SystemTime;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::types::Type;

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
    let line_end = text.chars().position(|c| c == '\n' || c == '\r');
    match line_end {
        None => (Some(text), Some("")),
        Some(i) => (Some(&text[..i]), Some(&text[i..])),
    }
}

pub async fn write_memo(
    pool: Arc<Pool>,
    memo: MemoWrite,
    security: Security,
) -> Result<GetWriteMemo, OrganizatorError> {
    let client = pool.get().await?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let millis: i64 = now.as_millis().try_into().unwrap();

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
                &memo.memoId,
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
    println!("{}", _stmt);

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
    pool: Arc<Pool>,
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