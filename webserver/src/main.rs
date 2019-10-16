#![feature(
  decl_macro,
  in_band_lifetimes,
  proc_macro_hygiene,
)]
#![recursion_limit = "1024"]

#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

mod backend;
mod config;
mod database;
mod errors;
mod i18n;
mod models;
mod redis_store;
mod routes;
mod sidekiq;
mod store;
mod utils;

use crate::routes::web::fairings;

use parking_lot::RwLock;

use rocket_contrib::templates::Template;

use tera::Tera;

use std::env;

pub static SERVER_VERSION: Option<&'static str> = include!(concat!(env!("OUT_DIR"), "/version"));

lazy_static! {
  pub static ref RESOURCES_VERSION: Option<String> = git2::Repository::open(".")
    .and_then(|r| r.revparse_single("HEAD").map(|p| p.id()))
    .map(|r| r.to_string())
    .ok();

  pub static ref EMAIL_TERA: Tera = {
    let path = env::var("EMAIL_TEMPLATES").expect("missing EMAIL_TEMPLATES environment variable");
    let mut tera = Tera::new(&path).expect("could not create tempating engine");
    tera.autoescape_on(vec![".html", ".htm", ".xml", ".html.tera"]);
    tera
  };

  pub static ref CAMO_URL: Option<url::Url> = env::var("CAMO_URL")
    .ok()
    .map(|x| url::Url::parse(&x).expect("CAMO_URL could not be parsed as a URL"));

  pub static ref CAMO_KEY: Vec<u8> = {
    let key = env::var("CAMO_KEY").expect("missing CAMO_KEY environment variable");
    key.into_bytes()
  };
}

fn main() {
  if sodiumoxide::init().is_err() {
    println!("could not initialize libsodium");
    return;
  }

  dotenv::dotenv().ok();

  let config_path = match env::args().nth(1) {
    Some(p) => p,
    None => {
      println!("please specify the path to the configuration file as the first argument");
      return;
    }
  };

  let config = match config::load_config(&config_path) {
    Ok(c) => RwLock::new(c),
    Err(e) => {
      println!("could not load config.toml: {}", e);
      return;
    }
  };

  lazy_static::initialize(&EMAIL_TERA);
  lazy_static::initialize(&CAMO_KEY);
  lazy_static::initialize(&CAMO_URL);

  rocket::ignite()
    .manage(database::init_pool())
    .manage(redis_store::init_pool())
    .manage(redis_store::init_sidekiq())
    .manage(config)
    .manage(reqwest::Client::new())
    .attach(fairings::Csp)
    .attach(fairings::SecurityHeaders)
    .attach(fairings::LastPage::default())
    .attach(fairings::Push)
    .attach(Template::custom(move |engines| {
      // FIXME: propagate error when possible
      let localisation = self::i18n::Localisation::new().unwrap();
      engines.tera.register_function("tr", self::i18n::tera_function(localisation));
    }))
    .register(catchers![
      routes::bad_request,
      routes::forbidden,
      routes::internal_server_error,
      routes::not_found,
    ])
    .mount("/", routes![
      routes::web::index::get,

      routes::web::about::get,

      routes::web::credits::get,

      routes::web::auth::login::get,
      routes::web::auth::login::post,

      routes::web::auth::logout::post,

      routes::web::auth::register::get,
      routes::web::auth::register::post,

      routes::web::pastes::get::id,
      routes::web::pastes::get::username_id,
      routes::web::pastes::get::users_username_id,

      routes::web::pastes::files::raw::get,

      routes::web::pastes::revisions::get,

      routes::web::pastes::get::edit,

      routes::web::pastes::post::post,

      routes::web::pastes::delete::delete,
      routes::web::pastes::delete::ids,
      routes::web::pastes::patch::patch,

      routes::web::account::index::get,
      routes::web::account::index::patch,
      routes::web::account::index::well_known_password_change,

      routes::web::account::keys::get,
      routes::web::account::keys::post,
      routes::web::account::keys::delete,

      routes::web::account::two_factor::get,
      routes::web::account::two_factor::enable_get,
      routes::web::account::two_factor::validate,
      routes::web::account::two_factor::disable_get,
      routes::web::account::two_factor::disable_post,
      routes::web::account::two_factor::new_secret,
      routes::web::account::two_factor::new_backup_codes,

      routes::web::account::delete::get,
      routes::web::account::delete::delete,

      routes::web::account::verify::get,
      routes::web::account::verify::resend,

      routes::web::account::reset_password::get,
      routes::web::account::reset_password::post,
      routes::web::account::reset_password::reset_get,
      routes::web::account::reset_password::reset_post,

      routes::web::account::avatar::get,

      routes::web::account::adminify::get,

      routes::web::admin::index::get,
      routes::web::admin::maintenance::get,
      routes::web::admin::pastes::get,
      routes::web::admin::users::get,

      routes::web::admin::config::get,
      routes::web::admin::config::post,

      routes::web::users::get::get,
    ])
    .mount("/static", routes!{
      routes::web::static_files::get,
    })
    .mount("/api/v1/pastes", routes![
      routes::api::pastes::get::get_all,

      routes::api::pastes::post::post_json,
      routes::api::pastes::post::post_multipart,
      routes::api::pastes::delete::delete,
      routes::api::pastes::delete::ids,
      routes::api::pastes::get::get,
      routes::api::pastes::patch::patch,

      routes::api::pastes::files::get::get,
      routes::api::pastes::files::patch::patch,
      routes::api::pastes::files::post::post,

      routes::api::pastes::files::individual::delete::delete,
      routes::api::pastes::files::individual::get::get,
      routes::api::pastes::files::individual::patch::patch,

      routes::api::pastes::files::individual::raw::get::get,
    ])
    .mount("/api/v0/pastes", routes![
      routes::api::pastes::get::get_all,

      routes::api::pastes::post::post_json,
      routes::api::pastes::delete::delete,
      routes::api::pastes::delete::ids,
      routes::api::pastes::get::get,
      routes::api::pastes::patch::patch,

      routes::api::pastes::files::get::get,
      routes::api::pastes::files::patch::patch,
      routes::api::pastes::files::post::post,

      routes::api::pastes::files::individual::delete::delete,
      routes::api::pastes::files::individual::get::get,
      routes::api::pastes::files::individual::patch::patch,

      routes::api::pastes::files::individual::raw::get::get,
    ])
    .mount("/api/v1/users", routes![
      routes::api::users::get::get,
    ])
    .mount("/api/v0/users", routes![
      routes::api::users::get::get,
    ])
    .launch();
}
