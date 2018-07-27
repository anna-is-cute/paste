use crate::{
  database::{
    DbConn,
    models::{files::File as DbFile, pastes::Paste as DbPaste},
    schema::{files, pastes},
  },
  errors::*,
  models::paste::output::OutputFile,
};

use super::FileId;

use diesel::prelude::*;

use unicase::UniCase;

use std::cmp::Ordering;

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

  pub fn output_files(&self, conn: &DbConn, paste: &DbPaste, with_content: bool) -> Result<Vec<OutputFile>> {
    let files = self.files(conn)?;
    let mut outputs: Vec<OutputFile> = files
      .into_iter()
      .map(|f| f.as_output_file(with_content, paste))
      .collect::<Result<_>>()?;

    let readme = UniCase::new("readme");
    outputs.sort_unstable_by(|a, b| {
      let a_name = match a.name {
        Some(ref n) => n,
        None => return a.name.cmp(&b.name),
      };
      let b_name = match b.name {
        Some(ref n) => n,
        None => return a.name.cmp(&b.name),
      };

      let a_readme = UniCase::new(a_name.split('.').next().expect("first split (a)")) == readme;
      let b_readme = UniCase::new(b_name.split('.').next().expect("first split (b)")) == readme;

      if a_readme && !b_readme {
        Ordering::Less
      } else if b_readme && !a_readme {
        Ordering::Greater
      } else {
        a_name.cmp(&b_name)
      }
    });

    Ok(outputs)
  }

  pub fn output_file(&self, conn: &DbConn, paste: &DbPaste, id: FileId, with_content: bool) -> Result<Option<OutputFile>> {
    let file = match self.file(conn, id)? {
      Some(f) => f,
      None => return Ok(None),
    };
    let output = file.as_output_file(with_content, paste)?;

    Ok(Some(output))
  }
}
