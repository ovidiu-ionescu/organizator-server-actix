use crate::{db, errors::OrganizatorError, password::{verify_password, compute_new_password, CREDENTIAL_LEN}};
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
use actix_session::Session;


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

#[derive(Deserialize)]
#[derive(Debug)]
pub struct LoginQuery {
    pub j_username: Option<String>,
    pub j_password: Option<String>,
}

#[post("/login")]
pub async fn login(
    login_query_form: Form<LoginQuery>,
    session: Session,
    db_pool: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let login_query = login_query_form.into_inner();
    let user_login = db::get_login(&db_pool.into_inner(), &login_query).await?;
    if verify_password(&login_query.j_password.unwrap(), &user_login) {
        session.set("username", login_query.j_username.unwrap())?;
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[get("/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse, OrganizatorError>{
    session.purge();
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct ChangePasswordQuery {
    pub username: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
}

impl ChangePasswordQuery {
    fn validate(&self) -> bool {
        return 
            self.old_password.is_some()
            && self.new_password.is_some()
    }
}

#[post("/change_password")]
pub async fn change_password(
    change_password_form: Form<ChangePasswordQuery>,
    security: Security,
    db_pool_data: Data<Pool>,
) -> Result<HttpResponse, OrganizatorError> {
    let change_password_form = change_password_form.into_inner();
    if !change_password_form.validate() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    // verify existing password, applies to already authenticated user
    let login_query = LoginQuery {
        j_username: Some(String::from(security.get_user_name())),
        j_password: None,
    };
    let db_pool = db_pool_data.into_inner();
    let user_login = db::get_login(&db_pool, &login_query).await?;
    if !verify_password(&change_password_form.old_password.unwrap(), &user_login) {
        return Ok(HttpResponse::BadRequest().finish());
    }
    // Only root can change the password of another user
    if user_login.id != 1 && change_password_form.username.is_some() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let target_username: &str = &change_password_form.username.as_ref().map(String::as_str).unwrap_or(security.get_user_name());

    // compute the new checksums
    let mut salt: Vec<u8> = Vec::with_capacity(CREDENTIAL_LEN);
    salt.resize(CREDENTIAL_LEN, 0u8);
    let mut pbkdf2_hash: Vec<u8> = Vec::with_capacity(CREDENTIAL_LEN);
    pbkdf2_hash.resize(CREDENTIAL_LEN, 0u8);
    compute_new_password(&change_password_form.new_password.unwrap(), &mut salt, &mut pbkdf2_hash)?;
    db::update_password(&db_pool, target_username, &salt, &pbkdf2_hash).await?;

    Ok(HttpResponse::NoContent().finish())
}
