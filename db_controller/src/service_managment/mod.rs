use diesel::*;

use std::time::SystemTime;

pub mod modules;
use modules::*;

impl crate::DbConn {
    pub fn register_session(&self, _user: &str , _service: i32, register_port: bool) -> Result<ActiveSession, diesel::result::Error> {
        use crate::schema::active_sessions::dsl::*;
        let conn = &mut *self.0.lock().unwrap();

        let _port: Option<i32> = if register_port {
            let used_ports: Vec<i32> = active_sessions.select(port.assume_not_null())
                .filter(port.is_not_null())
                .order(port.asc())
                .get_results(conn)?;

            let unused_port: Vec<i32> = (crate::SERVICE_PORT_RANGE.0..crate::SERVICE_PORT_RANGE.1)
                .filter(|x| !used_ports.contains(x))
                .collect();
            if unused_port.len() == 0 {
                return Err(diesel::result::Error::NotFound);
            }
            Some(unused_port[0])
        } else {
            None
        };

        let new_session = NewActiveSession {
            user: _user.to_string(),
            service_id: _service,
            port: _port,
            unix_created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
        };
        diesel::insert_into(active_sessions).values(&new_session).execute(conn).unwrap();
        Ok(active_sessions.order(id.desc()).first(conn)?)
    }

    pub fn add_docker_id(&self, _id: i32, _docker_id: &str, _container_ip: Option<&str>) -> Result<(), diesel::result::Error> {
        use crate::schema::active_sessions::dsl::*;
        let conn = &mut *self.0.lock().unwrap();

        let _ = diesel::update(active_sessions.filter(id.eq(_id)))
            .set((container_ip.eq(_container_ip), docker_id.eq(_docker_id)))
            .execute(conn)?;
        Ok(())
    }

    pub fn remove_session(&self, _user: &str, _service: i32) -> Result<(), diesel::result::Error> {
        use crate::schema::active_sessions::dsl::*;
        let conn = &mut *self.0.lock().unwrap();

        let _ = diesel::delete(active_sessions.filter(user.eq(_user)).filter(service_id.eq(_service))).execute(conn)?;
        Ok(())
    }

    pub fn get_sessions_by_user(&self, _user: &str) -> Result<Vec<ActiveSession>, diesel::result::Error> {
        let conn = &mut *self.0.lock().unwrap();
        use crate::schema::active_sessions::dsl::*;
        let result = active_sessions.filter(user.eq(_user)).get_results(conn)?;
        Ok(result)
    }

    pub fn get_service_by_id(&self, _id: i32) -> Result<Service, diesel::result::Error> {
        let conn = &mut *self.0.lock().unwrap();
        use crate::schema::services::dsl::*;
        let result = services.filter(id.eq(_id)).first(conn)?;
        Ok(result)
    }
}