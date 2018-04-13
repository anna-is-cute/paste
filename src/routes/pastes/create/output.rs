use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Success {
  id: String,
}

impl From<Uuid> for Success {
  fn from(id: Uuid) -> Self {
    let id = id.simple().to_string();
    Success { id }
  }
}
