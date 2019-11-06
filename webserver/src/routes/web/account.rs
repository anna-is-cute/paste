pub mod adminify;
pub mod avatar;
pub mod delete;
pub mod index;
pub mod keys;
pub mod reset_password;
pub mod two_factor;
pub mod verify;

use crate::routes::web::Links;

pub(crate) fn account_links() -> Links {
  links!(
    "settings" => uri!(crate::routes::web::account::index::get),
    "keys" => uri!(crate::routes::web::account::keys::get),
    "tfa" => uri!(crate::routes::web::account::two_factor::get),
    "delete_account" => uri!(crate::routes::web::account::delete::get),
  )
}
