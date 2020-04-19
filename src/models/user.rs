use serde::{Deserialize, Serialize};

use crate::crypto::hash;
use crate::schema::users;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel;
use diesel::SqliteConnection;

type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String, // hashed password of the user
}

#[derive(Serialize, Deserialize)]
pub struct SignIn {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignUp {
    pub password: String,
    pub password_retype: String,
    pub email: String,
}

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct UserForm {
    pub name: String,
    pub email: String,
    pub password: String, // hashed password
}

impl SignIn {
    pub fn new(email: String, password: String) -> SignIn {
        SignIn {
            email: email,
            password: password,
        }
    }

    pub fn hash_password(&mut self) -> () {
        let mut toks = self.password.split('&').fuse();
        let first = toks.next();
        match first {
            Some("hash") => return (),
            Some(_) => self.password = format!("{}{}", "hash&", hash::hash_sha256(&self.password)),
            None => panic!("should not happen!"),
        }
    }

    fn check_password_hashed(&self) -> bool {
        let mut toks = self.password.split('&').fuse();
        let first = toks.next();
        match first {
            Some("hash") => return true,
            Some(_) => return false,
            None => panic!("should not happen!"),
        }
    }
}

impl SignUp {
    fn check_password_matches(&self) -> bool {
        if self.password == self.password_retype {
            return true;
        }
        return false;
    }

    pub fn check_data_and_insert_to_database(
        mut self,
        pool: web::Data<Pool>,
    ) -> Result<UserForm, &'static str> {
        if !self.check_password_matches() {
            return Err("Passwords do not match.");
        }
        self.password = format!("{}{}", "hash&", hash::hash_sha256(&self.password));
        let form = UserForm {
            email: self.email,
            password: self.password,
            name: "".to_owned(),
        };
        return form.insert_user_to_database(pool);
    }
}

impl User {
    pub fn user_exists(user_email: &String, pool: web::Data<Pool>) -> bool {
        use self::diesel::prelude::*;
        use crate::schema::users::dsl::email;

        let conn: &SqliteConnection = &pool.get().unwrap();
        match users::table
            .filter(email.eq(user_email))
            .limit(1)
            .load::<User>(conn)
        {
            Ok(mut inside) => match inside.pop() {
                Some(_user) => return true,
                _ => return false,
            },
            _ => {
                println!("Error loading data in the Database!");
                return false;
            }
        };
    }

    pub fn get_user_from_email(user_email: &String, pool: &Pool) -> Option<Self> {
        use self::diesel::prelude::*;
        use crate::schema::users::dsl::email;

        let conn: &SqliteConnection = &pool.get().unwrap();
        match users::table
            .filter(email.eq(user_email))
            .limit(1)
            .load::<User>(conn)
        {
            Ok(mut inside) => match inside.pop() {
                Some(user) => return Some(user),
                _ => return None,
            },
            _ => {
                println!("Error loading data in the Database!");
                return None;
            }
        };
    }

    fn check_password_hashed(&self) -> bool {
        let mut toks = self.password.split('&').fuse();
        let first = toks.next();
        match first {
            Some("hash") => return true,
            Some(_) => return false,
            None => panic!("should not happen!"),
        }
    }

    /// Checks if a Username is in the database and the passwords are matching.
    ///
    /// # Arguments
    ///
    /// * `user` - A SignIn struct filled by the user.
    /// * `pool` - A poll to get database connections from.
    ///
    pub fn check_user_validity(user: &SignIn, pool: web::Data<Pool>) -> bool {
        use self::diesel::prelude::*;
        use crate::schema::users::dsl::email;

        let conn: &SqliteConnection = &pool.get().unwrap();
        let real_user = match users::table
            .filter(email.eq(&user.email))
            .limit(1)
            .load::<User>(conn)
        {
            Ok(mut inside) => match inside.pop() {
                Some(user) => user,
                _ => return false,
            },
            _ => {
                println!("Not found in Database!");
                return false;
            }
        };

        let pass = match user.check_password_hashed() {
            true => {
                if real_user.password == user.password {
                    println!("Pass is {}", user.password);
                    return true;
                }
                return false;
            }
            false => format!("{}{}", "hash&", hash::hash_sha256(&user.password)),
        };
        println!("Pass is {}", pass);

        if real_user.password == pass {
            return true;
        }
        return false;
    }
}

impl UserForm {
    fn insert_user_to_database(self, pool: web::Data<Pool>) -> Result<UserForm, &'static str> {
        use crate::schema::users::dsl::*;
        use diesel::prelude::*;

        if User::user_exists(&self.email, pool.clone()) {
            return Err("User already exists.");
        }

        let conn: &SqliteConnection = &pool.get().unwrap();
        diesel::insert_into(users)
            .values(&self)
            .execute(conn)
            .unwrap();
        return Ok(self);
    }
}
