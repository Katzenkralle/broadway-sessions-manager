
use rocket_dyn_templates::serde::Serialize;
use rocket_dyn_templates::Template;
use crate::login::AdminGuard;
use rocket::State;
use db_controller::DbConn;
use rocket_dyn_templates::context;

#[derive(Serialize)]
pub struct InviteKeyResponse {
    pub status: i32,
    pub key: Option<String>
}

use crate::OptionBoxes;

#[post("/createInvite")]
pub async fn create_invite_key(_admin: AdminGuard, db_conn: &State<DbConn>) -> Template {
    match db_conn.create_key() {
        Ok(created_key) => Template::render("partials/adminBoxOptions", context! {
            options: OptionBoxes::AdminInvite(created_key.inv_key)
        }),
        _ => Template::render("partials/adminBoxOptions", 
            context! {options:  OptionBoxes::AdminInvite("Could not Create key ".to_string())}),
    }
}

#[post("/delInvite")]
pub async fn delete_invite_key(_admin: AdminGuard, db_conn: &State<DbConn>) -> Template {
    match db_conn.del_all_keys() {
        Ok(()) => Template::render("partials/adminBoxOptions", context! {
            options: OptionBoxes::AdminInvite("All keys deleted".to_string())
        }),
        _ => Template::render("partials/adminBoxOptions", context! {
            options: OptionBoxes::AdminInvite("Could not delete keys".to_string())
        }),
    }
}

