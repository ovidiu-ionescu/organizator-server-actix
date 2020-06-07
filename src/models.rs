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


#[derive(Serialize, PostgresMapper)]
#[pg_mapper(table = "memo_group")]
pub struct MemoGroup {
    pub id: i32,
    pub name: String,
}

#[derive (Serialize)]
pub struct MemoGroupList {
    pub memogroups: Vec<MemoGroup>,
}

#[derive (Serialize)]
pub struct Memo {
    pub id:              i32,
    pub title:           Option<String>,
    pub memotext:        Option<String>,
    pub savetime:        Option<i64>,
    pub memogroup:       Option<MemoGroup>,
    pub user:            MemoUser,
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
        let group_id: Option<i32> = row.get("o_memo_group_id");
        let memo_group = match group_id {
            None => None,
            Some(id) => Some(MemoGroup {
                id: id,
                name: row.get("o_memo_group_name"),
            })
        };

        Self {
            memo: Memo {
                id: row.get("o_id"),
                title: row.get("o_title"),
                memotext: row.get("o_memotext"),
                savetime: row.get("o_savetime"),
                memogroup: memo_group,
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

#[derive (Serialize)]
pub struct GetWriteMemo {
    memo: Option<Memo>,
    user: MemoUser,
}

impl From<&Row> for GetWriteMemo {
    fn from(row: &Row) -> Self {
        // not all memos are assigned to groups
        let group_id: Option<i32> = row.get("io_memo_group_id");
        let memo_group = match group_id {
            None => None,
            Some(id) => Some(MemoGroup {
                id: id,
                name: row.get("o_memo_group_name"),
            })
        };

        // when a memo gets deleted we don't get the id back
        let memo_id: Option<i32> = row.get("io_memo_id");
        let memo = match memo_id {
            None => None,
            Some(id) => Some(Memo {
                id: id,
                title: row.get("io_memo_title"),
                memotext: row.get("io_memo_memotext"),
                savetime: row.get("io_savetime"),
                memogroup: memo_group,
                user: MemoUser {
                    id: row.get("o_user_id"),
                    name: row.get("o_username"),
                },
            })
        };

        Self {
            memo: memo,
            user: MemoUser {
                id: row.get("o_requester_id"),
                name: row.get("io_requester_name"),
            }
        }
    }
}

impl GetWriteMemo {
    pub fn from_row(row: &Row) -> Result<Self, Error> {
        Ok(Self::from(row))
    }
}

#[derive(Serialize, PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct Login {
    pub id: i32,
    pub username: Option<String>,
    pub salt: Vec<u8>,
    pub pbkdf2: Vec<u8>,
}

/*
 * File permissions is a bit overkill, most data coming back is not used
 */

#[derive (Serialize)]
pub struct GetFilePermissions {
    pub requester: MemoUser,
    pub user: MemoUser,
    pub o_memo_group_id: Option<i32>,
    pub access: i32,
}

impl From<&Row> for GetFilePermissions {
    fn from(row: &Row) -> Self {
        Self {
            requester: MemoUser {
                id: row.get("o_requester_id"),
                name: row.get("io_requester_name")
            },
            user: MemoUser {
                id: row.get("o_user_id"),
                name: row.get("o_username")
            },
            o_memo_group_id: row.get("o_memo_group_id"),
            access: row.get("o_access")
        }
    }
}

impl GetFilePermissions {
    pub fn from_row(row:&Row) -> Result<Self, Error> {
        Ok(Self::from(row))
    }
}

#[derive (Serialize, PostgresMapper)]
#[pg_mapper(table = "permissions")]
pub struct ExplicitPermission {
    pub memo_group_id:   i32,
    pub memo_group_name: Option<String>,
    pub user_group_id:   i32,
    pub user_group_name: Option<String>,
    pub user_id:         i32,
    pub username:        Option<String>,
    pub access:          i32,
}
