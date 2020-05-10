use crate::{db, errors::OrganizatorError};
use actix_web::{
    get, post,
    web::{Data, Form, Query},
    HttpResponse,
};
use deadpool_postgres::Pool;
use log::debug;
use serde::Deserialize;

use crate::check_security_middleware::Security;
use crate::models::{MemoGroupList, MemoTitleList, User};

#[derive(Deserialize)]
pub struct GetUserQuery {
    pub id: Option<i32>,
    pub username: Option<String>,
}

#[get("/user/{id}")]
pub async fn get_user(
    _qry: Query<GetUserQuery>,
    id: actix_web::web::Path<i32>,
    _security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    // println!("Query: {:#?} {:#?}", qry.id, qry.username);
    let query: GetUserQuery = GetUserQuery {
        id: Some(id.into_inner()),
        username: None,
    };
    let user = db::get_user(db_pool.into_inner(), query).await?;

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Deserialize)]
pub struct GetAllMemoTitlesQuery {
    pub username: Option<String>,
}

#[get("/memo/")]
pub async fn get_memo_titles(
    security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    debug!("Memos for user name {:#?}", security.user_name);

    let mut titles = db::get_memo_titles(security, db_pool.into_inner()).await?;
    let owner_entry = titles.pop().unwrap();
    let owner = User {
        id: owner_entry.user_id,
        username: owner_entry.title,
    };

    Ok(HttpResponse::Ok().json(MemoTitleList {
        memos: titles,
        user: owner,
    }))
}

#[derive(Deserialize)]
pub struct SearchMemoQuery {
    pub search: Option<String>,
}

#[post("/memo/search")]
pub async fn search_memo(
    qry: Form<SearchMemoQuery>,
    security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let query = qry.into_inner();
    debug!("Search memos with criteria {:#?}", &query.search);

    let mut titles = db::search_memo(db_pool.into_inner(), query, security).await?;
    let owner_entry = titles.pop().unwrap();
    let owner = User {
        id: owner_entry.user_id,
        username: owner_entry.title,
    };
    Ok(HttpResponse::Ok().json(MemoTitleList {
        memos: titles,
        user: owner,
    }))
}

#[get("/memo/{id}")]
pub async fn get_memo(
    id: actix_web::web::Path<i32>,
    security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let memo = db::get_memo(db_pool.into_inner(), id.into_inner(), security).await?;
    Ok(HttpResponse::Ok().json(memo))
}

#[derive(Deserialize)]
pub struct MemoWrite {
    pub memoId: Option<i32>,
    pub text: Option<String>,
    pub group_id: Option<i32>,
}
#[post("/memo/")]
pub async fn memo_write(
    memo_write: Form<MemoWrite>,
    security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let memo = db::write_memo(db_pool.into_inner(), memo_write.into_inner(), security).await?;
    Ok(HttpResponse::Ok().json(memo))
}

#[get("/memogroup/")]
pub async fn get_memo_group(
    security: Security,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let memogroups = db::get_memo_groups(db_pool.into_inner(), security).await?;
    Ok(HttpResponse::Ok().json(MemoGroupList {
        memogroups: memogroups,
    }))
}
