use crate::session::GTK3_SERVICE_ID;
use crate::session::GTK4_SERVICE_ID;

use db_controller::service_managment::modules::ActiveSession;
use db_controller::DbConn;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::{context, Template};
use rocket::State;
use rocket::form::Form;
use rocket::response::Redirect;

use rocket::request::{Outcome, Request, FromRequest};



fn get_element_from_back(query: &str, index: usize) -> Result<&str, ()> {
    // Split the query by slashes and collect into a vector of parts
    let parts: Vec<&str> = query.split('/').collect();

    // Check if there are at least two parts in the query
    if parts.len() < index {
        Err(()) // Return None if there are not enough parts
    } else {
        // Return the last or the penultimate part of the query depending on the parameter
        Ok(parts.get(parts.len() - index).unwrap_or(parts.last().unwrap()))
    }
}

pub fn port_in_origin(origin: &str) -> Result<i64, ()> {
    let last_part = get_element_from_back(origin, 1)?;
    if !last_part.len() < 5 && last_part.len() > 0 {
        return Err(())
    }
    Ok(last_part.parse::<i64>().or_else(|_| Err(()))?)
}



#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
    invite_key: Option<String>
}

#[derive(Clone)]
pub struct User {
    pub username: String,
    pub role: String,
}

#[derive(Clone)]
pub struct AdminGuard {
    pub user: User,
}


#[derive(Clone)]
pub struct SessionGuard {
    pub user: User,
    pub used_port: u32,
    pub used_ip: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_result = req.local_cache_async(async {
            let cookies = req.cookies();
            match cookies.get_private("session_id") {
                Some(_user) => {
                    let db_conn = req.guard::<&State<DbConn>>().await.unwrap();
                    return db_conn.get_user(_user.value(), None, None)
                        .and_then(|user_db| Ok(Outcome::Success(User {username: user_db.username,
                                role: user_db. role})))
                        .unwrap_or_else(|_| Outcome::Error((Status::Unauthorized, ())));
                },
                _ => Outcome::Error((Status::Unauthorized, ()))
            }
        }).await;
        user_result.to_owned()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_auth = req.guard::<User>().await;
        if user_auth.is_error(){
            return Outcome::Forward(Status::Unauthorized);
        }
        let user = user_auth.unwrap();
        if user.role == "admin" {
            return  Outcome::Success(AdminGuard{user: user});
        } else {
             return Outcome::Error((Status::Unauthorized, ()));
        }
        
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let port_auth_res = req.local_cache_async(async {
            let user_auth = req.guard::<User>().await;
            let db_conn = req.guard::<&State<DbConn>>().await.unwrap();

            if user_auth.is_error(){
                return Outcome::Forward(Status::Unauthorized);
            }
            let user = user_auth.unwrap();
            let headers = req.headers();
            let port: i32 = match headers.get("x-original-uri").nth(0){
                Some(ori_uri) => { 
                match port_in_origin(ori_uri) {
                    Ok(port) => port as i32,
                    Err(_) => {
                        let cookies = req.cookies();
                        let port = match cookies.get("session_port") {
                            Some(cookie) => match cookie.value().parse::<i64>() {
                                Ok(port) => port,
                                Err(_) => return Outcome::Forward(Status::Unauthorized),
                            },
                            None => return Outcome::Forward(Status::Unauthorized),
                        };
                        port as i32
                        }
                }},
                _ => return Outcome::Forward(Status::Unauthorized),
            };
            
            let client_ip = match req.cookies().get("session_destination")
                    .and_then(|cookie| Some(cookie.value().to_string())) {
                        Some(ip) => Some(ip),
                        None => {
                            match headers.get("x-original-uri").nth(0){
                                Some(ori_uri) => get_element_from_back(ori_uri, 2).ok()
                                    .and_then(|str| Some(str.to_string())),
                                None => None
                            }
                        }
                    };

                let sessions: Result<Vec<ActiveSession>, _> = db_conn.get_sessions_by_user(user.username.as_str());
                
                if let Ok(sessions) = sessions {
                    if let Some(session) = sessions.iter().find(|s| 
                        (s.service_id == GTK3_SERVICE_ID && s.port == Some(port) ||
                         s.service_id == GTK4_SERVICE_ID && s.port == Some(port)) &&
                        s.container_ip.as_deref() == client_ip.as_deref()
                    ) {
                        return Outcome::Success(SessionGuard { user: user, used_port: port as u32, used_ip: session.container_ip.clone().unwrap() });
                    }
                }
                return Outcome::Forward(Status::Unauthorized);
                
                
        }).await;
        port_auth_res.to_owned()
    }
}



#[post("/login", data = "<user>")]
pub fn login_post(user:Form<LoginForm>, cookies: &CookieJar<'_>, db_conn: &State<DbConn>) -> Result<Redirect, Template> { 
    match db_conn.get_user(&user.username, Some(&user.password), Some(false))
                    .and_then(|user_db| Ok(Ok(user_db)))
                    .unwrap_or_else(|_| Err(())) {
                    Ok(user) => {cookies.add_private(Cookie::new("session_id", user.username.clone()));},
                    _ => return Err(Template::render("pages/login", context! {err: "username or password incorrect"})),
                    }
    return Ok(Redirect::to(uri!(crate::index)));  
    }  

#[get("/login")]
pub fn login() -> Template {
    Template::render("pages/login", "")
}
#[get("/signup")]
pub async fn signup() -> Template {
    return Template::render("pages/signup", "")
}

#[post("/signup", data = "<user>")]
pub async fn signup_post(user: Form<LoginForm>, db_conn: &State<DbConn>) -> Result<Redirect, Template> {
    if user.invite_key.is_none(){
        return Err(Template::render("pages/signup", context! {err: "A Invitation key is required"}));
    };

    match db_conn.create_user(&user.username, &user.password, user.invite_key.as_ref().unwrap()) {
        Ok(()) => return Ok(Redirect::to(uri!(login))),
        Err(_) => return Err(Template::render("pages/signup", context! {err: "Could not create user, try another username!"}))
    }
}
