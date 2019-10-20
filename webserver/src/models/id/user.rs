uuid_wrapper!(UserId);

use diesel::prelude::*;

use crate::{
  database::{
    DbConn,
    models::users::User,
    schema::users,
  },
  errors::*,
};

impl UserId {
  pub fn get(&self, conn: &DbConn) -> Result<Option<User>> {
    Ok(users::table.find(self.0).first(&**conn).optional()?)
  }
}
