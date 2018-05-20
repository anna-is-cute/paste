use models::id::UserId;
use store::Store;

use sidekiq::{self, Value, JobOpts};

pub enum Job {
  DeleteAllPastes(UserId),
}

impl Job {
  fn class(&self) -> &str {
    match *self {
      Job::DeleteAllPastes(_) => "DeleteAllPastes",
    }
  }

  fn args(&self) -> Vec<sidekiq::Value> {
    match *self {
      Job::DeleteAllPastes(u) => {
        let path = Store::directory()
          .canonicalize()
          .expect("could not canonicalize store path")
          .join(u.simple().to_string())
          .to_string_lossy()
          .into_owned();
        vec![
          Value::String(path),
        ]
      },
    }
  }

  fn opts(&self) -> JobOpts {
    match *self {
      Job::DeleteAllPastes(_) => JobOpts {
        queue: "low".into(),
        .. Default::default()
      },
    }
  }
}

impl From<Job> for sidekiq::Job {
  fn from(j: Job) -> Self {
    sidekiq::Job::new(
      j.class().to_string(),
      j.args(),
      j.opts(),
    )
  }
}
