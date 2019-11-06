use md5::{Md5, Digest};

use data_encoding::HEXLOWER;

use diesel::{
  Queryable,
  backend::Backend,
  deserialize::{self, FromSql},
  serialize::{self, ToSql},
  sql_types::SmallInt,
};

use rocket::{http::RawStr, request::FromFormValue};

use sodiumoxide::crypto::hash::sha256;

use std::{
  borrow::Cow,
  cell::RefCell,
  io::Write,
};

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
      x => return Err(anyhow::anyhow!("bad admin enum: {}", x).into()),
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
  const LIBRAVATAR: (Cow<'static, str>, u16) = (Cow::Borrowed("seccdn.libravatar.org"), 443);

  pub fn domain(self, email: &str) -> (Cow<'static, str>, u16) {
    match self {
      AvatarProvider::Gravatar => (Cow::Borrowed("gravatar.com"), 443),
      AvatarProvider::Libravatar => {
        // get the email domain
        let domain = match email.split('@').last() {
          Some(d) => d,
          None => return AvatarProvider::LIBRAVATAR,
        };
        // query the secure avatars service
        let srv = match crate::RESOLV.lookup_srv(&format!("_avatars-sec._tcp.{}.", domain)) {
          Ok(s) => s,
          Err(_) => return AvatarProvider::LIBRAVATAR,
        };
        // filter for FQDNs
        let mut records: Vec<_> = srv.iter()
          .filter(|rec| rec.target().is_fqdn() && !rec.target().is_localhost())
          .collect();
        // sort by priority
        records.sort_by_key(|rec| rec.priority());
        // find the highest priority that we can resolve and is a global ip
        // note that this doesn't follow the SRV spec, but I don't really care
        records.iter()
          .filter(|rec| {
            let ip = match crate::RESOLV.lookup_ip(&rec.target().to_ascii()) {
              Ok(i) => i,
              Err(_) => return false,
            };
            ip.iter().all(|ip| ip.is_global())
          })
          .map(|rec| (Cow::Owned(rec.target().to_ascii()), rec.port()))
          .next()
          // otherwise, use default
          .unwrap_or(AvatarProvider::LIBRAVATAR)
      },
    }
  }

  pub fn hash(self, s: &str) -> String {
    match self {
      AvatarProvider::Gravatar => HEXLOWER.encode(&Md5::digest(s.as_bytes())[..]),
      AvatarProvider::Libravatar => HEXLOWER.encode(&sha256::hash(s.as_bytes())[..]),
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
      x => return Err(anyhow::anyhow!("bad avatar provider enum: {}", x).into()),
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
