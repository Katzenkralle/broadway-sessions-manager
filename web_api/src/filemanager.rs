use rocket_dyn_templates::{context, Template};
use rocket::State;
use rocket::serde::Serialize;
use rocket::request::{Outcome, Request, FromRequest};
use rocket::http::Status;
use rocket::fs::{FileName, TempFile};
use rocket::form::Form;
use rocket::tokio::task::spawn_blocking; 

use std::fs::{remove_dir_all, remove_file, create_dir};
use std::os::unix::fs::chown;
use std::path::{Path, PathBuf};

use crate::DbConn;
use crate::login::User as UserGuard;
use crate::NamedFile;
use crate::{ErrorTemplateContent, ErrorTemplateOption};


#[derive(Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
}

#[derive(Responder)]
pub enum FileResponder {
    Template(Template),
    File(Option<NamedFile>)
}

#[derive(Clone)]
pub struct ActiveSessionConstructorGuard {
    pub _id: Vec<i32>
}

#[derive(Serialize)]
struct FileResponse {
    dir_content: Vec<FileInfo>,
    parrent_dir_path: Option<String>,
    current: String,
    error_block: Option<ErrorTemplateContent>
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for ActiveSessionConstructorGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        
        let user_auth = req.guard::<crate::login::User>().await;
        if user_auth.is_error(){
            return Outcome::Error((Status::Unauthorized, ()));
        };
        let user = user_auth.unwrap();
        let db_conn = req.guard::<&State<DbConn>>().await.unwrap();
        match db_conn.get_sessions_by_user(&user.username) {
            Ok(session) => {
                if session.len() > 0 {
                    let construct = ActiveSessionConstructorGuard {
                        _id: session.iter().map(|elem| elem.id).collect()
                    };
                    return Outcome::Success(construct);
                }
            }
            _ => (),
        };
        return  Outcome::Error((Status::FailedDependency, ()));
        }
    }

fn get_session_root(_session: Option<ActiveSessionConstructorGuard>, username: &str) -> PathBuf {
    return Path::new("./data/user-home/").join(username);
}

fn strip_from_path(path: &PathBuf, subentrys: u32) -> PathBuf {
    let path_len = path.components().count();
    return PathBuf::from(&path
        .components()
        .take(path_len - subentrys as usize)
        .map(|comp| comp.as_os_str().to_str().unwrap())
        .collect::<Vec<_>>()
        .join("/"));
    }    

fn list_files(root_dir: &PathBuf, request_path: &PathBuf) -> Result<FileResponse, String> {
    // root_dir: ./active_sessions or ./data/user-home, request_path: [url after /files/]
    let mut dir_content: Vec<FileInfo>= vec![];
    let full_path_to_dir = root_dir.join(request_path);
    if request_path.to_str().is_none(){
        return Err("Cant resolve reqest Path".to_string());
    }

    // Try read dir, else exit funcition 
    for entry in full_path_to_dir.read_dir().map_err(|e| format!("Error reading directory: {}", e))? {
        if let Ok(entry) = entry {
            let filename = entry.file_name().into_string().map_err(|_| "Could not convert filename")?;
            if filename.chars().nth(0).unwrap_or_else(||'.') == '.' {
                continue;
            }
            dir_content.push(FileInfo {name: filename.clone(),
                    path: request_path.join(filename).to_str().unwrap().to_string(),
                    is_dir: entry.file_type().map_err(|_| "Could not determin if file is dir")?.is_dir()})
        }
    }
    return Ok(FileResponse {
        dir_content: dir_content,
        current: request_path.to_str().unwrap().to_string(),
        parrent_dir_path: match request_path.components().count() {
            0 => None,
            _ => Some("/files/".to_owned() + strip_from_path(&request_path, 1).to_str().unwrap())
        },
        error_block: None});

}


#[get("/files/<path..>")]
pub async fn display_files(path: PathBuf, user: UserGuard, session: Option<ActiveSessionConstructorGuard>) -> FileResponder {
    let root_path = get_session_root(session, &user.username);

    let fs_path = root_path.join(&path);

    if !fs_path.exists() {
        return FileResponder::Template(Template::render("pages/files", context! {error_msg: "The path dose not exist"}));
    }
    if !fs_path.is_dir() {
        // Use function on lower rank
        return FileResponder::File(NamedFile::open(&fs_path).await.ok());
    }
    let rendered_response = Template::render("pages/files", list_files(&root_path, &path)
        .unwrap_or_else(|err_msg| FileResponse {dir_content: vec![], current: path.to_str().unwrap().to_string(),
            parrent_dir_path: None,
            error_block: Some(ErrorTemplateContent {error: "An Error ocurred".to_string(), message: err_msg, options: None})}));

    FileResponder::Template(rendered_response)
}

#[post("/create_folder/<path..>", data = "<folder_name>")]
pub async fn create_folder(user: UserGuard, session: Option<ActiveSessionConstructorGuard>, folder_name: Form<String>, path: PathBuf)
    -> Template {
    let root_path = get_session_root(session, &user.username);
    let fs_path = root_path.join(&path);
    let target_dir = fs_path.join(folder_name.into_inner());
    //let uri_base = "/files/".to_string()+&path.to_str().unwrap();
    let dir_creation_result = create_dir(&target_dir);

    let mut folder_content = list_files(&root_path, &path).unwrap();
    match dir_creation_result {
        Ok(()) => {
            let _ = chown(&target_dir, Some(1000), Some(1000)).unwrap_or_else(|err| 
                folder_content.error_block = Some(ErrorTemplateContent {
                    error: "Could not change owner of Folder".to_string(),
                    message: format!("The folder was created but might not be writable. {}.", err).to_string(),
                    options: None
                }));
            return Template::render("pages/files", folder_content)
        },
        _ => {
            folder_content.error_block = Some(ErrorTemplateContent {
                error: "Could not create Folder".to_string(),
                message: "What would you like to do?".to_string(),
                options: Some(vec![ ErrorTemplateOption {location: "/files/".to_string(), name: "Back to root".to_string()}])
            });
            return Template::render("pages/files", folder_content);
        }
    }

}

async fn add_file_at(fs_path: &PathBuf, mut file: TempFile<'_>) -> Result<(), String> {
    println!("{:?}", file.raw_name());
    let filename = FileName::as_str(
        file.raw_name()
            .ok_or_else(|| String::from("Error getting filename"))?
    );

    if filename.is_none() {
        return Err("Illegale filename".to_string());
    }
   let target_file = fs_path.join(filename.unwrap());
    match file.move_copy_to(&target_file).await {
        Ok(_) => {
            return chown(&target_file, Some(1000), Some(1000))
                .map_err(|_| "Could not change owner of file".to_string());
         
        },
        _ => return Err("File could not save file".to_string()),
    } 
}

#[post("/upload/<path..>", data = "<file>")]
pub async fn upload_file(user: UserGuard, session: Option<ActiveSessionConstructorGuard>, file: Form<TempFile<'_>>, path: PathBuf) -> Template {
    let root_path = get_session_root(session, &user.username);

    let fs_path = root_path.join(&path);

    
    let result = add_file_at(&fs_path, file.into_inner()).await;
        
    let mut file_content = list_files(&root_path, &path).unwrap();
    if result.is_ok() {
        return Template::render("pages/files", file_content);
    } else {
        file_content.error_block = Some(ErrorTemplateContent {
            error: "Could not save file".to_string(),
            message: result.err().unwrap(),
            options: None
        });
        return Template::render("pages/files", file_content);
    }
}

async fn deleat_file_at(path: PathBuf) -> Result<(), String> {
    match spawn_blocking(move|| {
        if path.is_dir() {
            remove_dir_all(path)
        } else {
            remove_file(path)
        }
    }).await {
        Ok(_) => Ok(()),
        Err(_) => Err("Could not deleate file".to_string())
    }
}

#[post("/remove/<path..>")]
pub async fn deleate_file(user: UserGuard, session: Option<ActiveSessionConstructorGuard>, path: PathBuf) -> Template {
    let root_path = get_session_root(session, &user.username);

    let fs_path = root_path.join(&path);
    let path_len = path.components()
        .count(); 

    let parrent_dir_path = strip_from_path(&path, 1);

    let mut dir_content = list_files(&root_path, &parrent_dir_path).unwrap();

    if path_len < 1 {
        dir_content.error_block = Some(ErrorTemplateContent {
            error: "Cant remove root".to_string(),
            message: "What would you like to do?".to_string(),
            options: Some(vec![ ErrorTemplateOption {location: "/files/".to_string(), name: "Back to root".to_string()}])
        });
        return Template::render("pages/files", dir_content);
    }

   

    match deleat_file_at(fs_path.clone()).await {
        Ok(()) => {
            dir_content = list_files(&root_path, &parrent_dir_path).unwrap();
            return Template::render("pages/files", dir_content);
        },
        Err(msg) => {
            dir_content.error_block = Some(ErrorTemplateContent {
                error: "Could not deleate file".to_string(),
                message: msg,
                options: None
            });
            return Template::render("pages/files", dir_content);
        }
    }
}