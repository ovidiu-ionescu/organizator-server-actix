use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_session::UserSession;
use actix_web::{
    dev::{Extensions, Payload, ServiceRequest, ServiceResponse},
    Error, FromRequest, HttpMessage, HttpRequest,
    error:: {ErrorUnauthorized}
};
use futures::future::{ok, Ready};
use futures::Future;

use log::debug;

#[derive(Debug, Clone)]
pub struct Security {
    pub user_name: Option<String>,
    // pub user_name: usize,
}

impl Security {
    pub fn from(s: &str) -> Security {
        Security {
            user_name: Some(String::from(s)),
        }
        // Security{ user_name: 2}
    }

    fn get_security(extensions: &mut Extensions) -> Security {
        match extensions.get::<Security>() {
            Some(s) => s.clone(),
            None => {
                println!("No security in here");
                Security::from("aha")
            }
        }
    }
}

impl FromRequest for Security {
    type Error = Error;
    type Future = Ready<Result<Security, Error>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ok(Security::get_security(&mut *req.extensions_mut()))
    }
}

impl Security {
    pub fn get_user_name(&self) -> &str {
        match &self.user_name {
            Some(s) => &s,
            None => "",
        }
    }
}

pub struct CheckSecurity;

// Middleware factory
impl<S, B> Transform<S> for CheckSecurity
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckSecurityMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckSecurityMiddleware { service })
    }
}

pub struct CheckSecurityMiddleware<S> {
    service: S,
}

impl<S, B> Service for CheckSecurityMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if req.path() != "/login" {
            let user_name_res = req.get_session().get::<String>("username");
            let mut found_user = false;
            if let Ok(o) = user_name_res {
                if o.is_some() {
                    let username = o.unwrap();
                    req.extensions_mut().insert(Security::from(&username));
                    found_user = true;
                    debug!("Found the username in the session {}", &username);
                }
            }
            if !found_user {
                debug!("No user found in the session, look up the certificate header");
                let a = req.headers().get("X-SSL-Client-S-DN");
                if a.is_some() {
                    let user_name = a.unwrap().to_str().unwrap();
                    debug!("X-SSL-Client-S-DN {}", user_name);
                    req.extensions_mut().insert(Security::from(&user_name[3..]));
                    found_user = true;
                }
            }
            if !found_user {
                debug!("No username found in either session or header, return unathorised");
                return  Box::pin(async move {
                    let res = req.error_response(ErrorUnauthorized("Unauthorised"));
                    Ok(res)
                });
            }
        }
        //println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            //println!("Hi from response");
            Ok(res)
        })
    }
}
