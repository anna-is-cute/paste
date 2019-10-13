use md5::{Md5, Digest};

use diesel::{
  Queryable,
  backend::Backend,
  deserialize::{self, FromSql},
  serialize::{self, ToSql},
  sql_types::SmallInt,
};

use rocket::{http::RawStr, request::FromFormValue};

use sodiumoxide::crypto::hash::sha256;

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

#[derive(Debug, Clone, Copy, PartialEq, AsExpression, Serialize, Deserialize)]
#[sql_type = "SmallInt"]
#[serde(rename_all = "lowercase")]
pub enum AvatarProvider {
  Gravatar,
  Libravatar,
}

impl AvatarProvider {
  pub fn domain(self) -> &'static str {
    match self {
      AvatarProvider::Gravatar => "gravatar.com",
      AvatarProvider::Libravatar => "seccdn.libravatar.org",
    }
  }

  pub fn hash(self, s: &str) -> String {
    match self {
      AvatarProvider::Gravatar => hex::encode(&Md5::digest(s.as_bytes())[..]),
      AvatarProvider::Libravatar => hex::encode(&sha256::hash(s.as_bytes())[..]),
    }
  }
}

impl Default for AvatarProvider {
  fn default() -> Self {
    AvatarProvider::Gravatar
  }
}

impl<DB: Backend<RawValue = [u8]>> Queryable<SmallInt, DB> for AvatarProvider {
  type Row = i16;

  fn build(row: Self::Row) -> Self {
    match row {
      0 => AvatarProvider::Gravatar,
      1 => AvatarProvider::Libravatar,
      _ => panic!("invalid avatar provider in database"),
    }
  }
}

impl<DB: Backend> ToSql<SmallInt, DB> for AvatarProvider {
  fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
    let prov: i16 = match *self {
      AvatarProvider::Gravatar => 0,
      AvatarProvider::Libravatar => 1,
    };

    <i16 as ToSql<SmallInt, DB>>::to_sql(&prov, out)
  }
}

impl<DB: Backend<RawValue = [u8]>> FromSql<SmallInt, DB> for AvatarProvider {
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    let visibility = match <i16 as FromSql<SmallInt, DB>>::from_sql(bytes)? {
      0 => AvatarProvider::Gravatar,
      1 => AvatarProvider::Libravatar,
      x => return Err(Box::new(format_err!("bad avatar provider enum: {}", x).compat())),
    };
    Ok(visibility)
  }
}

impl FromFormValue<'v> for AvatarProvider {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
      let vis = match form_value.as_str() {
        "gravatar" => AvatarProvider::Gravatar,
        "libravatar" => AvatarProvider::Libravatar,
        _ => return Err(form_value),
      };

      Ok(vis)
    }

    fn default() -> Option<Self> {
      Some(Default::default())
    }
}
