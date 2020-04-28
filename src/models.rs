use serde::Serialize;
use tokio_pg_mapper::PostgresMapper;

#[derive(Serialize, PostgresMapper)]
#[pg_mapper(table = "user")]
pub struct User {
    pub id: i32,
    pub username: Option<String>,
}

#[derive(Serialize, PostgresMapper)]
#[pg_mapper(table = "memo")]
pub struct MemoTitle {
    pub id: i32,
    pub title: Option<String>,
    pub user_id: i32,
    pub savetime: Option<i64>,
}

#[derive (Serialize)]
pub struct MemoTitleList {
    pub memos: Vec<MemoTitle>,
    pub user: User,
}