use diesel::{
  Queryable,
  backend::Backend,
  deserialize::{self, FromSql},
  serialize::{self, ToSql},
  sql_types::SmallInt,
};

use failure::format_err;

use std::io::Write;

/// Admin status of a [`User`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, AsExpression)]
#[sql_type = "SmallInt"]
#[serde(rename_all = "lowercase")]
pub enum Admin {
  /// User is not an admin.
  None,
  /// User is a normal admin.
  Normal,
  /// User is a superadmin.
  Super,
}

impl Default for Admin {
  fn default() -> Self {
    Admin::None
  }
}

impl<DB: Backend<RawValue = [u8]>> Queryable<SmallInt, DB> for Admin {
  type Row = i16;

  fn build(row: Self::Row) -> Self {
    match row {
      0 => Admin::None,
      1 => Admin::Normal,
      2 => Admin::Super,
      _ => panic!("invalid admin in database")
    }
  }
}

impl<DB: Backend> ToSql<SmallInt, DB> for Admin {
  fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
    let admin: i16 = match *self {
      Admin::None => 0,
      Admin::Normal => 1,
      Admin::Super => 2,
    };

    <i16 as ToSql<SmallInt, DB>>::to_sql(&admin, out)
  }
}

impl<DB: Backend<RawValue = [u8]>> FromSql<SmallInt, DB> for Admin {
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    let admin = match <i16 as FromSql<SmallInt, DB>>::from_sql(bytes)? {
      0 => Admin::None,
      1 => Admin::Normal,
      2 => Admin::Super,
      x => return Err(Box::new(format_err!("bad admin enum: {}", x).compat())),
    };
    Ok(admin)
  }
}
