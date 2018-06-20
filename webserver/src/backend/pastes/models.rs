use backend::errors::BackendError;
use database::models::deletion_keys::DeletionKey;
use database::models::files::File;
use database::models::pastes::Paste;
use database::models::users::User;
use models::paste::{Content, Visibility};

use failure::Error;

pub struct PastePayload<'u> {
  pub name: Option<String>,
  pub description: Option<String>,
  pub visibility: Visibility,
  pub author: Option<&'u User>,
  pub files: Vec<FilePayload>,
}

pub struct FilePayload {
  pub name: Option<String>,
  pub content: Content,
}

pub struct CreateSuccess {
  pub paste: Paste,
  pub files: Vec<File>,
  pub deletion_key: Option<DeletionKey>,
}

pub enum CreateError {
  NoFiles,
  AnonymousPrivate,
  DuplicateFileNames,
  PasteNameTooLarge,
  PasteNameTooLong,
  PasteDescriptionTooLarge,
  PasteDescriptionTooLong,
  FileNameTooLarge,
  FileNameTooLong,
  EmptyFile,
  Internal(Error),
}

impl BackendError for CreateError {
  fn into_message(self) -> Result<&'static str, Error> {
    let m = match self {
      CreateError::Internal(e) => return Err(e),
      CreateError::NoFiles => "you must upload at least one file",
      CreateError::AnonymousPrivate => "cannot make anonymous private pastes",
      CreateError::DuplicateFileNames => "duplicate file names are not allowed",
      CreateError::PasteNameTooLarge => "paste name must be less than 25 KiB",
      CreateError::PasteNameTooLong => "paste name must be less than or equal to 255 characters",
      CreateError::PasteDescriptionTooLarge => "paste description must be less than 25 KiB",
      CreateError::PasteDescriptionTooLong => "paste description must be less than or equal to 255 characters",
      CreateError::FileNameTooLarge => "file name must be less than 25 KiB",
      CreateError::FileNameTooLong => "file name must be less than or equal to 255 characters",
      CreateError::EmptyFile => "file content must not be empty",
    };

    Ok(m)
  }
}
