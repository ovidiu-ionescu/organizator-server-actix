use serde::Serialize;
use tokio_pg_mapper::PostgresMapper;
use tokio_postgres::row::Row;
use tokio_postgres::error::Error;

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

#[derive (Serialize)]
pub struct MemoGroup {
    pub id: i32,
    pub name: String,
}

#[derive (Serialize)]
pub struct Memo {
    pub id:              i32,
    pub title:           Option<String>,
    pub memotext:        Option<String>,
    pub savetime:        i64,
    pub memo_group: MemoGroup,
    pub user: MemoUser,
}

#[derive (Serialize)]
pub struct MemoUser {
    pub id: i32,
    pub name: Option<String>,
}

#[derive (Serialize)]
pub struct GetMemo {
    memo: Memo,
    user: MemoUser,
}

impl From<&Row> for GetMemo {
    fn from(row: &Row) -> Self {
        Self {
            memo: Memo {
                id: row.get("o_id"),
                title: row.get("o_title"),
                memotext: row.get("o_memotext"),
                savetime: row.get("o_savetime"),
                memo_group: MemoGroup {
                    id: row.get("o_memo_group_id"),
                    name: row.get("o_memo_group_name"),
                },
                user: MemoUser {
                    id: row.get("o_user_id"),
                    name: row.get("o_username"),
                },
            },
            user: MemoUser {
                id: row.get("o_requester_id"),
                name: row.get("o_requester_name"),
            }
        }
    }
}

impl GetMemo {
    pub fn from_row(row: &Row) -> Result<Self, Error> {
        Ok(Self::from(row))
    }
}