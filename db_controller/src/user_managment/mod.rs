use diesel::*;

pub mod models;
use models::*;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::SystemTime;

use password_auth::{generate_hash, verify_password, VerifyError};

impl crate::DbConn {
    pub fn create_user<'a>(&self, usr: &'a str, passwd: &'a str, key: &str) ->  Result<(), diesel::result::Error> {
        use crate::schema::users;
        use crate::schema::invite_key::dsl::*;
        let conn = &mut *self.0.lock().unwrap();
        
        let _: InviteKey = invite_key.find(key).first(conn)?;

        let hash = generate_hash(passwd);
        let created_user = NewUser {
            username: usr.to_string(),
            password: Some(hash),
            role: "user".to_string(),
        };
    
        diesel::insert_into(users::table)
            .values(&created_user)
            .execute(conn)?;
        diesel::delete(invite_key.find(key)).execute(conn).unwrap();
        Ok(())
    }
    
    pub fn update_user(&self, usr: &str, new_username: Option<&str>, new_passwd: Option<&str>) -> Result<(), diesel::result::Error> {
        use crate::schema::users;
        let conn = &mut *self.0.lock().unwrap();
        if let Some(new_username) = new_username {
            diesel::update(users::table.find(usr))
                .set(users::username.eq(new_username))
                .execute(conn)?;
        }
        if let Some(new_passwd) = new_passwd {
            let hash = generate_hash(new_passwd); 
            diesel::update(users::table.find(usr))
                .set(users::password.eq(Some(hash)))
                .execute(conn)?;
        }
        Ok(())
    }
    
    pub fn remove_user(&self, usr: &str) -> Result<(), diesel::result::Error> {
        use crate::schema::users::dsl::*;
        let conn = &mut *self.0.lock().unwrap();
        diesel::delete(users.filter(username.eq(usr)))
            .execute(conn)?;
        Ok(())
    }
    
    pub fn get_user(&self, usr: &str, passwd: Option<&str>, as_hash: Option<bool>) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;
    
        let conn = &mut *self.0.lock().unwrap();
        let query = users.filter(username.eq(usr));
        if let Some(passwd) = passwd {

            let hash:Option<String> = query.select(password).get_result(conn)?;
            let matching_passwd = match as_hash {
                Some(true) => if passwd.to_string() == hash.unwrap_or_else(|| " ".to_string())
                    {Ok(())} else {Err(VerifyError::PasswordInvalid)},
                _ => verify_password(passwd, &hash.unwrap_or_else(|| " ".to_string()))
            };
                
            match matching_passwd {
                Ok(_) => (),
                Err(_) => return Err(diesel::result::Error::NotFound),
            }
        }
    
        let user = query.first(conn)?;
        Ok(user)
    }
    
    pub fn create_key(&self) -> Result<InviteKey, diesel::result::Error> {
        use crate::schema::invite_key::dsl::*;
        let conn = &mut *self.0.lock().unwrap();
        
        loop {
            let key_str: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            // a must be hear for some reason..
            let a:Result<InviteKey, diesel::result::Error> = invite_key.filter(inv_key.eq(&key_str)).first(conn);
            if a.is_err() {
                let key = InviteKey {
                    inv_key: key_str,
                    unix_created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                };
    
                diesel::insert_into(invite_key)
                    .values(&key)
                    .execute(conn)?;
                return Ok(key);
            }
        }
    }

    pub fn key_present(&self, key: &str) -> bool {
        use crate::schema::invite_key::dsl::*;
        let conn = &mut *self.0.lock().unwrap();
        
        let findings: Result<InviteKey, diesel::result::Error> = invite_key.filter(inv_key.eq(key)).first(conn);
        match findings {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn del_all_keys(&self) -> Result<(), ()> {
        use crate::schema::invite_key::dsl::*;
        let conn = &mut *self.0.lock().unwrap();

        match diesel::delete(invite_key).execute(conn) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(()) 
        }
    }
}
