use crate::models::paste::{Content, Paste, PasteFile};

use mime::{Mime, TopLevel};

use multipart::server::Multipart;

use rocket::{
  Request, Data, Outcome,
  http::Status,
  data::{self, FromDataSimple},
};

use std::io::Read;

lazy_static! {
  static ref JSON: Mime = "application/json".parse().unwrap();
}

pub struct MultipartUpload(pub Paste);

impl std::ops::Deref for MultipartUpload {
  type Target = Paste;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl MultipartUpload {
  pub fn into_inner(self) -> Paste {
    self.0
  }
}

impl FromDataSimple for MultipartUpload {
  type Error = String;

  fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
    let boundary = match request.content_type() {
      Some(ct) if ct.is_form_data() => {
        match ct.params().find(|&(key, _)| key == "boundary").map(|(_, value)| value) {
          Some(b) => b,
          None => return Outcome::Forward(data),
        }
      },
      _ => return Outcome::Forward(data),
    };

    if !request.headers().contains("X-Paste") {
      return Outcome::Forward(data);
    }

    let mut mp = Multipart::with_body(data.open(), boundary);

    let mut entry = match mp.read_entry() {
      Ok(Some(e)) => e,
      Ok(None) => return Outcome::Failure((Status::BadRequest, "expected at least one multipart file".into())),
      Err(e) => return Outcome::Failure((Status::InternalServerError, e.to_string())),
    };

    if entry.headers.content_type != Some(JSON.clone()) {
      return Outcome::Failure((Status::BadRequest, "expected first multipart file to be application/json metadata".into()));
    }

    let mut paste: Paste = match serde_json::from_reader(&mut entry.data) {
      Ok(j) => j,
      Err(e) => return Outcome::Failure((Status::BadRequest, e.to_string())),
    };

    let mut files = Vec::with_capacity(1);
    loop {
      match entry.next_entry_inplace() {
        Ok(None) => break,
        Err(e) => return Outcome::Failure((Status::InternalServerError, e.to_string())),
        _ => {},
      }

      let mut data = Vec::new();
      if let Err(e) = entry.data.read_to_end(&mut data) {
        return Outcome::Failure((Status::InternalServerError, e.to_string()));
      }

      let content = match entry.headers.content_type {
        Some(Mime(TopLevel::Image, _, _)) => Content::Base64(data),
        _ => match String::from_utf8(data) {
          Ok(s) => Content::Text(s),
          Err(e) => Content::Base64(e.into_bytes()),
        },
      };

      let name = if &*entry.headers.name == "" {
        None
      } else {
        Some(entry.headers.name.to_string().into())
      };

      files.push(PasteFile {
        name,
        highlight_language: None, // FIXME
        content,
      });
    }

    paste.files.extend(files);

    Outcome::Success(MultipartUpload(paste))
  }
}
