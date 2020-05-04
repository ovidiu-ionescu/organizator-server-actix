use crate::{errors::OrganizatorError, models::{ User, MemoTitle, GetMemo}};
use crate::routes::{ GetUserQuery, GetAllMemoTitlesQuery, SearchMemoQuery,  };
use deadpool_postgres::Pool;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::types::Type;
use std::sync::Arc;
use crate::check_security_middleware::Security;

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
    let stmt = client.prepare_typed("select * from get_memo($1, $2);", &[Type::INT4, Type::VARCHAR]).await.unwrap();

    client.query(
        &stmt,
        &[&id, &String::from(&security.user_name.clone().unwrap())]
    ).await?
    .iter_mut()
    .map(move |row| GetMemo::from_row(row).map_err(OrganizatorError::from))
    .next().unwrap()
}