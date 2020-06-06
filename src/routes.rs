use crate::{
    db,
    errors::OrganizatorError,
    password::{compute_new_password, verify_password, CREDENTIAL_LEN},
};
use actix_web::{
    get, post, web,
    web::{Data, Form, Query},
    HttpRequest, HttpResponse,
};
use deadpool_postgres::Pool;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::check_security_middleware::Security;
use crate::models::{MemoGroupList, MemoTitleList, User};
use actix_multipart::Multipart;
use actix_session::Session;
use futures::{StreamExt, TryStreamExt};

use std::io::Write;

use uuid::Uuid;
use std::str::FromStr;
use crate::config::{FileUploadConfig };


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

#[derive(Deserialize, Debug)]
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
pub async fn logout(session: Session) -> Result<HttpResponse, OrganizatorError> {
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
        return self.old_password.is_some() && self.new_password.is_some();
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

    let target_username: &str = &change_password_form
        .username
        .as_ref()
        .map(String::as_str)
        .unwrap_or(security.get_user_name());

    // compute the new checksums
    let mut salt: Vec<u8> = Vec::with_capacity(CREDENTIAL_LEN);
    salt.resize(CREDENTIAL_LEN, 0u8);
    let mut pbkdf2_hash: Vec<u8> = Vec::with_capacity(CREDENTIAL_LEN);
    pbkdf2_hash.resize(CREDENTIAL_LEN, 0u8);
    compute_new_password(
        &change_password_form.new_password.unwrap(),
        &mut salt,
        &mut pbkdf2_hash,
    )?;
    db::update_password(&db_pool, target_username, &salt, &pbkdf2_hash).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/version")]
pub async fn version() -> Result<HttpResponse, OrganizatorError> {
    Ok(HttpResponse::Ok()
        .content_type("plain/text")
        .header("X-Version", "sample")
        .body("1.0.2\n"))
}

#[derive(Deserialize)]
pub struct UploadFileQuery {
    pub id: Option<i32>,
}

fn extension(filename: &str) -> Option<&str> {
    let dot = filename.rfind('.');
    match dot {
        Some(i) => Some(&filename[i..]),
        None => None,
    }
}

fn without_extension(filename: &str) -> &str {
    let dot = filename.rfind('.').unwrap_or(filename.len());
    let slash = filename.rfind('/');
    match slash {
        Some(s) => &filename[s + 1 .. dot],
        None => &filename[0 .. dot]
    }
}

#[cfg(test)]
mod test_extension {
    #[test]
    fn test_extension() {
        assert_eq!(super::extension("aha.txt"), Some(".txt"));
        assert_eq!(super::extension("no dot"), None);
        assert_eq!(super::extension("more dots...txt"), Some(".txt"));
    }

    #[test]
    fn test_filename() {
        assert_eq!(super::without_extension("file.txt"), "file");
        assert_eq!(super::without_extension("/path/file.txt"), "file");
        assert_eq!(super::without_extension("file..txt"), "file.");
    }
}

#[derive(Serialize)]
struct FileUpload {
    filename: String,
}

#[post("/upload")]
pub async fn upload_file(
    mut payload: Multipart, 
    file_upload_config_data: Data<FileUploadConfig>,
    security: Security,
    db_pool_data: Data<Pool>,

) -> Result<HttpResponse, OrganizatorError> {
    let db_pool = db_pool_data.into_inner();


    let file_upload_config = file_upload_config_data.into_inner();

    let mut memo_group_id: Option<i32> = None;

    let mut res = FileUpload{ filename: String::new() };

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();
        // if let Some(filename) = content_type.get_filename() {
        match content_type.get_filename() {
            Some(filename) => {
                // extract the extension
                let file_uuid = Uuid::new_v4();
                let ext = extension(&filename);
                let filepath = format!("{}/{}{}", file_upload_config.dir, file_uuid, ext.unwrap_or(""));
                res = FileUpload { filename: format!("{}{}", file_uuid, ext.unwrap_or("")) };
                

                // let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));
                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| std::fs::File::create(filepath))
                    .await
                    .unwrap();
                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&data).map(|_| f)).await?;
                }
                debug!("memo_group_id for file: {:#?}", &memo_group_id);
                // add the database entry
                db::insert_filestore(&db_pool, &file_uuid, security.get_user_name(), &filename, &memo_group_id).await?;
            }
            None => {
                let mut val = String::with_capacity(20);
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    val.push_str(&String::from_utf8(data.to_vec())?);
                }
                if name == "memo_group_id" && val.len() > 0 {
                    memo_group_id = Some(val.parse::<i32>().unwrap());
                debug!("just parsed memo_group_id for file: {:#?}", &memo_group_id);
                }
                println!("Parameter {}, value {}", name, &val);
            }
        }
        // let filename = content_type.get_filename().unwrap();
    }
    Ok(HttpResponse::Ok().json(res))
}

#[get("/file_auth")]
pub async fn file_auth(
    request: HttpRequest,
    security: Security,
    db_pool: Data<Pool>,

) -> Result<HttpResponse, OrganizatorError> {
    let req_headers = request.headers();
    let uri = req_headers.get("X-Original-URI").unwrap().to_str().unwrap();
    debug!("Checking security for file {}", uri);
    let filename = without_extension(uri);

    let uuid = Uuid::from_str(&filename).unwrap();
    let _permissions = db::file_permissions(&db_pool.into_inner(), &uuid, security.get_user_name(), Some(1)).await?;
    Ok(HttpResponse::NoContent().finish())
    //Ok(HttpResponse::NoContent().finish())
    //Ok(HttpResponse::Unauthorized().finish())
}

