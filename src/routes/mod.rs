use errors::Result;
use models::status::Status;

use rocket::response::status::Custom;

use rocket_contrib::Json;

pub type RouteResult<T> = Result<Custom<Json<Status<T>>>>;

pub mod pastes;
