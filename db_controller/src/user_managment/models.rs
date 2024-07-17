use diesel::prelude::*;

#[derive(Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub username: String,
    pub password: Option<String>,
    pub role: String,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password: Option<String>,
    pub role: String
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = crate::schema::invite_key)]

pub struct InviteKey{
    pub inv_key: String,
    pub unix_created_at: i64,
}
