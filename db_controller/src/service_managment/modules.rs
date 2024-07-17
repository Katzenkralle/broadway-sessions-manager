use diesel::prelude::*; // This is the trait that provides the select and insert methods


#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::services)]

#[allow(dead_code)]
pub struct Service {
    id: i32,
    name: String,
    description: Option<String>,
}


#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::active_sessions)]
pub struct ActiveSession {
    pub id: i32,
    pub user: String,
    pub service_id: i32,
    pub docker_id: Option<String>,
    pub container_ip: Option<String>,
    pub port: Option<i32>,
    pub unix_created_at: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::active_sessions)]
pub struct NewActiveSession {
    pub user: String,
    pub service_id: i32,
    pub port: Option<i32>,
    pub unix_created_at: i64
}
/*
romSqlRow<(diesel::sql_types::Integer, diesel::sql_types::Text, diesel::sql_types::Integer, diesel::sql_types::Nullable<diesel::sql_types::Integer>, diesel::sql_types::Nullable<diesel::sql_types::Timestamp>), Sqlite>`
*/

