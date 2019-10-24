uuid_wrapper!(SessionId);

impl SessionId {
  pub fn redis_key(self) -> String {
    format!("session:{}", self.0.to_simple())
  }
}
