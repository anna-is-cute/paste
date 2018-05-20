macro_rules! uuid_wrapper {
  ($(#[$meta:meta])* $name:ident) => {
    $(#[$meta])*
    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, FromSqlRow, AsExpression)]
    #[sql_type = "::diesel::pg::types::sql_types::Uuid"]
    pub struct $name(pub ::uuid::Uuid);

    impl From<::uuid::Uuid> for $name {
      fn from(id: ::uuid::Uuid) -> Self {
        $name(id)
      }
    }

    impl ::std::ops::Deref for $name {
      type Target = ::uuid::Uuid;

      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl $name {
      #[allow(unused)]
      pub fn into_inner(self) -> ::uuid::Uuid {
        self.0
      }
    }

    impl ::serde::Serialize for $name {
      fn serialize<S>(&self, ser: S) -> ::std::result::Result<S::Ok, S::Error>
        where S: ::serde::Serializer,
      {
        self.0.simple().to_string().serialize(ser)
      }
    }

    impl<'de> ::serde::Deserialize<'de> for $name {
      fn deserialize<D>(des: D) -> ::std::result::Result<Self, D::Error>
        where D: ::serde::Deserializer<'de>
      {
        ::uuid::Uuid::deserialize(des).map($name)
      }
    }

    impl<'a> ::rocket::request::FromParam<'a> for $name {
      type Error = &'a ::rocket::http::RawStr;

      fn from_param(param: &'a ::rocket::http::RawStr) -> ::std::result::Result<Self, &'a ::rocket::http::RawStr> {
        use ::std::str::FromStr;
        match ::uuid::Uuid::from_str(param) {
          Ok(u) => Ok(u.into()),
          Err(_) => Err(param)
        }
      }
    }

    impl ::diesel::serialize::ToSql<::diesel::sql_types::Uuid, ::diesel::pg::Pg> for $name {
      fn to_sql<W: ::std::io::Write>(&self, out: &mut ::diesel::serialize::Output<W, ::diesel::pg::Pg>) -> ::diesel::serialize::Result {
        <::uuid::Uuid as ::diesel::serialize::ToSql<::diesel::sql_types::Uuid, ::diesel::pg::Pg>>::to_sql(&self.0, out)
      }
    }

    impl<A> ::diesel::deserialize::FromSql<A, ::diesel::pg::Pg> for $name {
      fn from_sql(bytes: Option<&[u8]>) -> ::diesel::deserialize::Result<Self> {
        ::uuid::Uuid::from_sql(bytes).map($name)
      }
    }

    impl ::std::fmt::Display for $name {
      fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0.simple())
      }
    }
  }
}

pub mod api_key;
pub mod deletion_key;
pub mod file;
pub mod paste;
pub mod session;
pub mod user;

pub use self::api_key::ApiKeyId;
pub use self::deletion_key::DeletionKeyId;
pub use self::file::FileId;
pub use self::paste::PasteId;
pub use self::session::SessionId;
pub use self::user::UserId;
