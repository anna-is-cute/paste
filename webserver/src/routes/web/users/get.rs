use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{users, pastes};
use errors::*;
use models::paste::output::{Output, OutputAuthor, OutputFile};
use models::paste::Visibility;
use routes::web::{context, Rst, OptionalWebUser, Session};

use diesel::dsl::count;
use diesel::prelude::*;

use rocket::State;
use rocket::http::Status as HttpStatus;

use rocket_contrib::Template;

use std::result;

#[get("/users/<username>")]
fn get(username: String, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(1, username, config, user, sess, conn)
}

#[get("/users/<username>?<params>")]
fn get_page(username: String, params: PageParams, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(params.page, username, config, user, sess, conn)
}

#[derive(Debug, FromForm)]
struct PageParams {
  page: u32,
}

fn _get(page: u32, username: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  // TODO: make PositiveNumber struct or similar (could make Positive<num::Integer> or something)
  if page == 0 {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }
  let target: User = match users::table.filter(users::username.eq(&username)).first(&*conn).optional()? {
    Some(u) => u,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let mut query = DbPaste::belonging_to(&target)
    .select(count(pastes::id))
    .into_boxed();
  if Some(target.id()) != user.as_ref().map(|x| x.id()) {
    query = query.filter(pastes::visibility.eq(Visibility::Public));
  }
  let total_pastes: i64 = query.get_result(&*conn)?;

  let outputs = if total_pastes == 0 && page == 1 {
    Vec::default()
  } else {
    let page = i64::from(page);
    let offset = (page - 1) * 15;
    if offset >= total_pastes {
      return Ok(Rst::Status(HttpStatus::NotFound));
    }
    let pastes: Vec<DbPaste> = if Some(target.id()) == user.as_ref().map(|x| x.id()) {
      DbPaste::belonging_to(&target)
        .order_by(pastes::created_at.desc())
        .offset(offset)
        .limit(15)
        .load(&*conn)?
    } else {
      DbPaste::belonging_to(&target)
        .filter(pastes::visibility.eq(Visibility::Public))
        .order_by(pastes::created_at.desc())
        .offset(offset)
        .limit(15)
        .load(&*conn)?
    };

    let author = OutputAuthor::new(target.id(), target.username(), target.name());

    let mut outputs = Vec::with_capacity(pastes.len());

    for paste in pastes {
      let id = paste.id();

      let files: Vec<OutputFile> = id.files(&conn)?
        .iter()
        .map(|x| x.as_output_file(false, &paste))
        .collect::<result::Result<_, _>>()?;

      outputs.push(Output::new(
        paste.id(),
        Some(author.clone()),
        paste.name(),
        paste.description(),
        paste.visibility(),
        None,
        files,
      ));
    }

    outputs
  };

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["pastes"] = json!(outputs);
  ctx["target"] = json!(target);
  ctx["page"] = json!(page);
  ctx["total"] = json!(total_pastes);
  Ok(Rst::Template(Template::render("user/index", ctx)))
}
