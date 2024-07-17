
use db_controller::DbConn;
//use rocket::tokio::time::{sleep, Duration};


#[macro_use] extern crate rocket;
// Alternativly iport rocket manually: use rocket::get;
use rocket_dyn_templates::{context, Template};
use rocket::fs::FileServer;
use rocket::response::Redirect;
use rocket::fs::NamedFile;
use rocket::State;
use std::env;
use docker_api::docker::Docker;

mod login;
use login::User;
use session::GtkBoxOptions;
mod session;
mod filemanager;
mod user_actions;
use rocket_dyn_templates::serde::Serialize;

#[derive(Serialize)]
pub struct ErrorTemplateOption {
    location: String,
    name: String,
}

#[derive(Serialize)]
pub struct ErrorTemplateContent {
    error: String,
    message: String,
    options: Option<Vec<ErrorTemplateOption>>,
}

#[derive(Serialize)]
struct Box {
    headline: String,
    description: String,
    bg_color: String,
    start_request_to: Option<String>,
    option_box: String,
    options: OptionBoxes
}

#[derive(Serialize)]
pub enum OptionBoxes {
    GtkSession(GtkBoxOptions),
    AdminInvite(String)
}

struct DockerSettings {
    shared_volume: String,
    vroot_image: String,
    common_network: Option<String>,
}

#[get("/", rank = 1)]
async fn index(user: Option<User>, db_conn: &State<DbConn>) -> Result<Template, Redirect> {
    match user {
        Some(user) => {
            let render_admin_settings = user.role == "admin";

            let active_sessions = session::status(
                db_conn,
                &user,
                Some(vec![
                    session::GTK3_SERVICE_ID,
                    session::GTK4_SERVICE_ID,
                ]))
                .await;

            let mut boxes = vec![Box{
                headline: "Broadway".to_string(),
                description: "Use Broadway sessions to work with Desktop GTK 3/4 applications from your browser".to_string(),
                bg_color: "#a6d189".to_string(),
                start_request_to: Some("/api/start/gtk".to_string()),
                option_box: "partials/broadwayBoxOptions".to_string(),
                options: OptionBoxes::GtkSession(GtkBoxOptions::find_status(active_sessions))
            }];

            if user.role == "admin" {
                boxes.push(Box{
                    headline: "Admin: Invite Keys".to_string(),
                    description: "Create Invite keys".to_string(),
                    bg_color: "#ea999c".to_string(),
                    option_box: "partials/adminBoxOptions".to_string(),
                    start_request_to: None,
                    options: OptionBoxes::AdminInvite("".to_string())
                });
            }
            
            Ok(Template::render(
                "pages/home",
                context! {
                    username: user.username,
                    role: user.role,
                    admin_settings: render_admin_settings,
                    boxes: boxes,
                },
            ))
        },
        None => Err(Redirect::to(uri!(login::login))),
    }
}

#[get("/", rank = 2)]
fn login_redirect() -> Redirect {
    return Redirect::to(uri!(login::login));
}

#[get("/selector")] // Mounted on /selector_template, gets renderd by js
async fn selector() -> Option<NamedFile> {
    return NamedFile::open("web_api/public/templates/partials/selector.html.hbs").await.ok();
}


#[catch(401)]
fn unauth() -> Redirect {
    Redirect::to("/login")
}

#[catch(404)]
fn not_found() -> Redirect {
    Redirect::to("/")
}

#[launch]
fn rocket() -> _ {
    //Init docker settings
    let docker_settings: DockerSettings = DockerSettings {
        shared_volume: env::var("SHARED_VOLUME_NAME").expect("You must provide a SHARED_VOLUME_NAME env"),
        vroot_image: env::var("VROOT_IMAGE").expect("You must provide a VROOT_IMAGE env"),
        common_network: match env::var("COMMON_NETWORK"){
            Ok(network) => if network != "" { Some(network) } else { None }, 
            _ => None,
        }
    };

    println!("{:?}", env::temp_dir().display());
    rocket::build()
        .manage(DbConn::establish_connection()) // Manage the state here
        .manage(Docker::new("unix:///var/run/docker.sock").unwrap())
        .manage(docker_settings)
        .attach(Template::fairing())
        // Serve files from `/www/static` at path `/public`
        .mount("/static", FileServer::from("web_api/public/static/"))
        .mount("/", routes![login::login, login::login_post, login::signup, login::signup_post])
        .mount("/", routes![index, login_redirect, filemanager::display_files])
        .mount("/api", routes![session::start, session::end, session::handover, 
            user_actions::create_invite_key, user_actions::delete_invite_key,
            filemanager::deleate_file, filemanager::create_folder, filemanager::upload_file])
        .mount("/dyn-template", routes![selector])
        .register("/", catchers![not_found, unauth])
        //.register("/", catchers![unauth])
}