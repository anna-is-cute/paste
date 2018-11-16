use crate::{
  config::Config,
  errors::*,
  models::id::UserId,
  store::Store,
};

use failure::format_err;

use serde::Serialize;

use sidekiq::{Value, JobOpts};

use std::path::PathBuf;

pub enum Job<'c> {
  DeleteAllPastes(&'c Config, UserId),
  Email {
    config_path: PathBuf,
    email: String,
    subject: String,
    content: String,
  },
  Queue {
    class: String,
    timestamp: i64,
    args: Vec<Value>,
  },
}

impl Job<'c> {
  pub fn email<T, C, P, E, S>(template: T, context: C, path: P, email: E, subject: S) -> Result<Job<'c>>
    where T: AsRef<str>,
          C: Serialize,
          P: Into<PathBuf>,
          E: Into<String>,
          S: Into<String>,
  {
    let rendered = crate::EMAIL_TERA
      .render(template.as_ref(), &context)
      .map_err(|e| format_err!("tera error: {}", e))?;

    Ok(Job::Email {
      config_path: path.into(),
      email: email.into(),
      subject: subject.into(),
      content: rendered,
    })
  }

  pub fn queue<C: Into<String>>(class: C, timestamp: i64, args: Vec<Value>) -> Job<'c> {
    Job::Queue {
      class: class.into(),
      timestamp,
      args,
    }
  }

  fn class(&self) -> &str {
    match *self {
      Job::DeleteAllPastes(_, _) => "DeleteDirectory",
      Job::Email { .. } => "Email",
      Job::Queue { .. } => "Queue",
    }
  }

  fn args(&self) -> Vec<Value> {
    match *self {
      Job::DeleteAllPastes(config, u) => {
        let path = Store::new(config)
          .directory()
          .join(u.to_simple().to_string())
          .to_string_lossy()
          .into_owned();
        vec![
          Value::String(path),
        ]
      },
      Job::Email { ref config_path, ref email, ref subject, ref content } => vec![
        Value::String(config_path.to_string_lossy().into_owned()),
        Value::String(email.to_string()),
        Value::String(subject.to_string()),
        Value::String(content.to_string()),
      ],
      Job::Queue { ref class, timestamp, ref args } => vec![
        Value::String(class.to_string()),
        Value::Number(timestamp.into()),
        Value::Array(args.clone()),
      ],
    }
  }

  fn opts(&self) -> JobOpts {
    match *self {
      Job::DeleteAllPastes(_, _) | Job::Queue { .. } => JobOpts {
        queue: "low".into(),
        .. Default::default()
      },
      Job::Email { .. } => Default::default(),
    }
  }
}

impl From<Job<'c>> for sidekiq::Job {
  fn from(j: Job) -> Self {
    sidekiq::Job::new(
      j.class().to_string(),
      j.args(),
      j.opts(),
    )
  }
}
