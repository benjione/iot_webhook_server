use diesel;
use diesel::SqliteConnection;

use actix_web::web;
use serde::{Deserialize, Serialize};

use chrono::Utc;

use crate::schema::devices;

type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Device {
    pub id: i32,                 // id of the device, unique
    pub registration_id: String, // Unique ID generated for registration purposes
    pub name: String,            // name of the device
    pub user: i32,               // User it belongs to
    pub online: bool,            // Determines if the device is online
    pub registration_date: String,
    pub last_login: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "devices"]
pub struct NewDevice {
    pub registration_id: String,
    pub name: String,
    pub user: i32,
    pub online: bool,
    pub registration_date: String,
    pub last_login: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewDeviceForm {
    pub name: String,
}

impl Device {
    pub fn device_exists(user_id: i32, object_name: String, pool: &web::Data<Pool>) -> bool {
        use self::diesel::prelude::*;
        use crate::schema::devices::dsl::{name, user};

        let conn: &SqliteConnection = &pool.get().unwrap();
        match devices::table
            .filter(user.eq(user_id))
            .filter(name.eq(object_name))
            .limit(1)
            .load::<Device>(conn)
        {
            Ok(mut inside) => match inside.pop() {
                Some(_) => return true, // object found in database
                _ => return false,
            },
            _ => {
                return false;
            }
        };
    }

    pub fn get_devices_for_user(user_id: i32, pool: &web::Data<Pool>) -> Vec<Self> {
        use self::diesel::prelude::*;
        use crate::schema::devices::dsl::{name, user};

        let conn: &SqliteConnection = &pool.get().unwrap();
        let devices = devices::table.filter(user.eq(user_id)).load::<Device>(conn);
        match devices {
            Ok(ret) => return ret,
            Err(_) => return Vec::<Device>::new(),
        }
    }

    pub fn get_device(user_id: i32, object_name: String, pool: &Pool) -> Option<Self> {
        use self::diesel::prelude::*;
        use crate::schema::devices::dsl::{name, user};

        let conn: &SqliteConnection = &pool.get().unwrap();
        match devices::table
            .filter(user.eq(user_id))
            .filter(name.eq(object_name))
            .limit(1)
            .load::<Device>(conn)
        {
            Ok(mut inside) => match inside.pop() {
                Some(device) => return Some(device), // object found in database
                _ => return None,
            },
            _ => {
                return None;
            }
        };
    }

    pub fn get_device_and_register(user_id: i32, object_name: String, pool: &Pool) -> Option<Self> {
        use self::diesel::prelude::*;
        use crate::schema::devices::dsl::{name, online, user};
        match Device::get_device(user_id, object_name.clone(), pool) {
            Some(device) => {
                let conn: &SqliteConnection = &pool.get().unwrap();
                let _ = diesel::update(
                    devices::table
                        .filter(user.eq(user_id))
                        .filter(name.eq(object_name)),
                )
                .set(online.eq(true))
                .execute(conn);
                return Some(device);
            }
            _ => return None,
        }
    }

    pub fn go_offline(device_id: &String, pool: &Pool) -> () {
        use self::diesel::prelude::*;
        use crate::schema::devices::dsl::{online, registration_id};
        let conn: &SqliteConnection = &pool.get().unwrap();
        let _ = diesel::update(devices::table.filter(registration_id.eq(device_id)))
            .set(online.eq(false))
            .execute(conn);
    }
}

impl NewDevice {
    pub fn new(form: NewDeviceForm, user_id: i32) -> Self {
        use uuid::Uuid;
        NewDevice {
            registration_id: Uuid::new_v4().to_string(),
            name: form.name,
            user: user_id,
            online: false,
            registration_date: Utc::now().to_string(),
            last_login: "None".to_string(),
        }
    }
    pub fn insert_into_database(self, pool: web::Data<Pool>) {
        use crate::schema::devices::dsl::*;
        use diesel::prelude::*;
        let conn: &SqliteConnection = &pool.get().unwrap();
        diesel::insert_into(devices)
            .values(&self)
            .execute(conn)
            .unwrap();
    }
}
