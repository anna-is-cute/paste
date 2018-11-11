use crate::{
  database::models::users::User,
  models::id::PasteId,
  routes::web::Links,
};

pub mod delete;
pub mod files;
pub mod get;
pub mod patch;
pub mod post;
pub mod revisions;

pub(crate) fn paste_links(id: PasteId, author_name: &str, user: Option<&User>) -> Links {
  let mut links = links!(
    "files" => uri!(crate::routes::web::pastes::get::users_username_id: author_name, id),
    "revisions" => uri!(crate::routes::web::pastes::revisions::get: author_name, id),
    "delete" => uri!(crate::routes::web::pastes::delete::delete: author_name, id),
    "author_page" => uri!(crate::routes::web::users::get::get: author_name),
  );
  if let Some(ref u) = user {
    links.add(
      "edit",
      uri!(crate::routes::web::pastes::get::edit: u.username(), id),
    );
  }
  links
}
