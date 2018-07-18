use crate::{
  database::{
    DbConn,
    models::{files::File as DbFile, pastes::Paste as DbPaste},
    schema::{files, pastes},
  },
  errors::*,
};

use super::FileId;

use diesel::prelude::*;

uuid_wrapper!(
  /// An ID for a paste, which may or may not exist.
  ///
  /// Mostly useful for having Rocket accept only valid IDs in routes.
  PasteId
);

impl PasteId {
  pub fn len(&self, conn: &DbConn) -> Result<usize> {
    let size: i64 = files::table
      .filter(files::paste_id.eq(self.0))
      .select(diesel::dsl::count(files::id))
      .first(&**conn)?;

    Ok(size as usize)
  }

  pub fn is_empty(&self, conn: &DbConn) -> Result<bool> {
    Ok(self.len(conn)? == 0)
  }

  pub fn next_generic_name(&self, conn: &DbConn) -> Result<String> {
    Ok(format!("pastefile{}", self.len(conn)? + 1))
  }

  pub fn get(&self, conn: &DbConn) -> Result<Option<DbPaste>> {
    Ok(pastes::table.find(self.0).first(&**conn).optional()?)
  }

  pub fn files(&self, conn: &DbConn) -> Result<Vec<DbFile>> {
    Ok(files::table.filter(files::paste_id.eq(self.0)).load(&**conn)?)
  }

  pub fn file(&self, conn: &DbConn, id: FileId) -> Result<Option<DbFile>> {
    Ok(files::table
      .find(id)
      .filter(files::paste_id.eq(self.0))
      .first(&**conn)
      .optional()?)
  }
}
