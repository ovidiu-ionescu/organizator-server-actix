use crate::{errors::OrganizatorError, models::{ User, MemoTitle, GetMemo, GetWriteMemo}};
use crate::routes::{ GetUserQuery, GetAllMemoTitlesQuery, SearchMemoQuery, MemoWrite };
use deadpool_postgres::Pool;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::types::Type;
use std::sync::Arc;
use crate::check_security_middleware::Security;
use std::time::SystemTime;
use std::convert::TryInto;

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
    client.query(&stmt, &[&query.id]).await?
        .iter()
        .map(|row| User::from_row_ref(row).map_err(OrganizatorError::from))
        .next().unwrap()
}

impl GetAllMemoTitlesQuery {
    pub fn get_statement(&self) -> &'static str {
        include_str!("sql/get_all_memo_titles.sql")
    }
}
pub async fn get_memo_titles(pool: Arc<Pool>, query: GetAllMemoTitlesQuery) -> Result<Vec<MemoTitle>, OrganizatorError> {
    let _stmt = query.get_statement();

    let client = pool.get().await?;
    let stmt = client.prepare_typed(&_stmt, &[Type::VARCHAR]).await.unwrap();

    client.query(
        &stmt,
        &[ &query.username.unwrap()]
    ).await?
    .iter()
    .map(|row| MemoTitle::from_row_ref(row).map_err(OrganizatorError::from))
    .collect()
}


impl SearchMemoQuery {
    pub fn get_statement(&self) -> &'static str {
        include_str!("sql/search_memo.sql")
    }
}

pub async fn search_memo(pool: Arc<Pool>, query: SearchMemoQuery, security: Security) -> Result<Vec<MemoTitle>, OrganizatorError> {
    let sql = query.get_statement();

    let client = pool.get().await?;
    let stmt = client.prepare_typed(&sql, &[Type::VARCHAR]).await.unwrap();


    client.query(
        &stmt,
        &[ &String::from(&security.user_name.clone().unwrap()), &query.search.unwrap()]
    ).await?
    .iter()
    .map(|row| MemoTitle::from_row_ref(row).map_err(OrganizatorError::from))
    .collect()

}

pub async fn get_memo(pool: Arc<Pool>, id: i32, security: Security) -> Result<GetMemo, OrganizatorError>{
    let client = pool.get().await?;
    let stmt = client.prepare_typed("select * from memo_read($1, $2);", &[Type::INT4, Type::VARCHAR]).await.unwrap();

    client.query(
        &stmt,
        &[&id, &String::from(&security.user_name.clone().unwrap())]
    ).await?
    .iter_mut()
    .map(move |row| GetMemo::from_row(row).map_err(OrganizatorError::from))
    .next().unwrap()
}

fn split_memotekst<'a>(text: &'a str) -> (Option<&'a str>, Option<&'a str>) {
    let line_end = text.chars().position(|c| c == '\n' || c == '\r');
    match line_end {
        None => (Some(text), Some("")),
        Some(i) => (Some(&text[..i]), Some(&text[i..]))
    }
}

pub async fn write_memo(pool: Arc<Pool>, memo: MemoWrite, security: Security) -> Result<GetWriteMemo, OrganizatorError> {
    let client = pool.get().await?;
    
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
    let millis: i64 = now.as_millis().try_into().unwrap();

    let stmt = client.prepare_typed("select * from memo_write($1, $2, $3, $4, $5, $6);", 
        &[Type::INT4, Type::VARCHAR, Type::VARCHAR, Type::INT8, Type::INT4, Type::VARCHAR]).await.unwrap();
    
    
    let s = memo.text.unwrap_or("".to_string());
    let b = split_memotekst(&s);

    client.query(
        &stmt,
        &[
            &memo.memoId,
            &b.0,
            &b.1,
            &millis,
            &memo.group_id,
            &String::from(&security.user_name.clone().unwrap())
        ]
    ).await?
    .iter_mut()
    .map(move |row| GetWriteMemo::from_row(row).map_err(OrganizatorError::from))
    .next().unwrap()
}