use actix_web::{get, post, web::{Query, Form, Data}, HttpResponse };
use crate::{errors::OrganizatorError, db};
use deadpool_postgres::Pool;
use serde::Deserialize;
use log::{debug};

use crate::check_security_middleware::Security;
use crate::models::{ User, MemoTitleList};

#[derive(Deserialize)]
pub struct GetUserQuery {
    pub id: Option<i32>,
    pub username: Option<String>
}

#[get("/user/{id}")]
pub async fn get_user(
    _qry: Query<GetUserQuery>,
    _security: Security,
    id: actix_web::web::Path<i32>, db_pool: Data<Pool>
) -> Result<HttpResponse, OrganizatorError> {
    
    // println!("Query: {:#?} {:#?}", qry.id, qry.username);
    let query: GetUserQuery = GetUserQuery{id: Some(id.into_inner()), username: None}; 
    let user = db::get_user(db_pool.into_inner(), query).await?;

    Ok(HttpResponse::Ok().json(user))
}


#[derive(Deserialize)]
pub struct GetAllMemoTitlesQuery {
    pub username: Option<String>,
}

#[get("/memo")]
pub async fn get_memo_titles(
    security: Security,
    db_pool: Data<Pool>
) -> Result<HttpResponse, OrganizatorError> {

    debug!("Memos for user name {:#?}", security.user_name);

    let gmt = GetAllMemoTitlesQuery { username: security.user_name.clone() };
    let mut titles = db::get_memo_titles(db_pool.into_inner(), gmt).await?;
    let owner_entry = titles.pop().unwrap();
    let owner = User { id: owner_entry.user_id, username: owner_entry.title};

    Ok(HttpResponse::Ok().json(MemoTitleList {memos: titles, user: owner }))
}

#[derive(Deserialize)]
pub struct SearchMemoQuery {
    pub search: Option<String>,
}

#[post("/memo")]
pub async fn search_memo (
    qry: Form<SearchMemoQuery>,
    security: Security,
    db_pool: Data<Pool>
) -> Result<HttpResponse, OrganizatorError> {
    let query = qry.into_inner();
    debug!("Search memos with criteria {:#?}", &query.search);

    let mut titles = db::search_memo(db_pool.into_inner(), query, security).await?;
    let owner_entry = titles.pop().unwrap();
    let owner = User { id: owner_entry.user_id, username: owner_entry.title};
    Ok(HttpResponse::Ok().json(MemoTitleList { memos: titles, user: owner }))
}