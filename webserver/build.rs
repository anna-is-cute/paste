#![feature(crate_visibility_modifier)]

use git2::Repository;

use std::{
  fs::File,
  io::Write,
  path::Path,
};

fn main() {
  write_version_file();
  credits::credits();
}

fn write_version_file() {
  let to_write = match version() {
    Some(v) => format!("Some(\"{}\")", v),
    None => "None".into(),
  };

  let out_dir = std::env::var("OUT_DIR").unwrap();
  let p = Path::new(&out_dir);
  let mut f = File::create(p.join("version")).unwrap();
  f.write_all(to_write.as_bytes()).unwrap();
}

fn version() -> Option<String> {
  let repo = Repository::open("..").ok()?;
  let revparse = repo.revparse_single("HEAD").ok()?;
  Some(revparse.id().to_string())
}

crate mod credits {
  use serde_derive::{Deserialize, Serialize};

  use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
  };

  #[derive(Debug, Deserialize)]
  crate struct Credits {
    #[serde(skip)]
    _path: Option<PathBuf>,
    people: Vec<Person>,
    patrons: Vec<Patron>,
    frontend: Vec<Dependency>,
    backend: Vec<Dependency>,
  }

  #[derive(Debug, Deserialize, Serialize)]
  crate struct Person {
    name: String,
    github: String,
    text: String,
  }

  #[derive(Debug, Deserialize, Serialize)]
  crate struct Patron {
    name: String,
    url: Option<String>,
    text: String,
  }

  #[derive(Debug, Deserialize, Serialize)]
  crate struct Dependency {
    name: String,
    url: String,
    text: String,
  }

  impl Credits {
    fn load() -> Self {
      let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

      for parent in Path::new(&manifest_dir).ancestors() {
        let credits_path = parent.join("credits").join("credits.toml");
        if credits_path.exists() {
          let credits_str = std::fs::read_to_string(&credits_path)
            .expect(&format!("couldn't read {:?} to string", credits_path));

          let mut credits: Credits = toml::from_str(&credits_str).unwrap();
          credits._path = Some(credits_path);
          credits.frontend.sort_by_key(|dep| dep.name.to_lowercase());
          credits.backend.sort_by_key(|dep| dep.name.to_lowercase());

          return credits;
        }
      }

      panic!("no credits.toml found")
    }

    fn check_all_deps(&self) {
      let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

      let manifest_str = std::fs::read_to_string(Path::new(&manifest_dir).join("Cargo.toml")).unwrap();
      let manifest: super::manifest::Manifest = toml::from_str(&manifest_str).unwrap();

      for dep in manifest.dependencies.keys() {
        assert!(
          manifest.package.metadata.credits.ignore.contains(dep) || self.backend.iter().any(|x| x.name == *dep),
          "{} isn't in credits.toml",
          dep,
        );
      }

      for dep in &self.backend {
        if !manifest.dependencies.contains_key(&dep.name) {
          eprintln!("warning: {} is in credits.toml and perhaps shouldn't be", dep.name);
        }
      }
    }

    fn create_html(&self) {
      #[derive(Serialize)]
      struct Section<'a> {
        name: &'static str,
        cols: Vec<&'a [String]>,
      }

      let glob = format!("{}/*.html.tera", self._path.as_ref().and_then(|x| x.parent()).unwrap().to_string_lossy());
      let tera = tera::Tera::new(&glob).unwrap();

      let mut final_html = String::new();

      // people

      let people_html: Vec<String> = self.people
        .iter()
        .map(|person| {
          let mut ctx = serde_json::to_value(person).unwrap();
          ctx["is_team"] = serde_json::json!(true);
          tera.render("credit.html.tera", &ctx).unwrap()
        })
        .collect();

      let people_section = tera.render("section.html.tera", &Section {
        name: "People",
        cols: people_html
          .chunks((people_html.len() as f32 / 3.0).ceil() as usize)
          .collect(),
      }).unwrap();

      final_html.push_str(&people_section);

      // patrons

      if !self.patrons.is_empty() {
        let patron_html: Vec<String> = self.patrons
          .iter()
          .map(|person| tera.render("credit.html.tera", person).unwrap())
          .collect();

        let patron_section = tera.render("section.html.tera", &Section {
          name: "Patrons",
          cols: patron_html
            .chunks((patron_html.len() as f32 / 3.0).ceil() as usize)
            .collect(),
        }).unwrap();

        final_html.push_str(&patron_section);
      }

      // frontend

      let frontend_html: Vec<String> = self.frontend
        .iter()
        .map(|person| tera.render("credit.html.tera", person).unwrap())
        .collect();

      let frontend_section = tera.render("section.html.tera", &Section {
        name: "Frontend",
        cols: frontend_html
          .chunks((frontend_html.len() as f32 / 3.0).ceil() as usize)
          .collect(),
      }).unwrap();

      final_html.push_str(&frontend_section);

      // backend

      let backend_html: Vec<String> = self.backend
        .iter()
        .map(|person| tera.render("credit.html.tera", person).unwrap())
        .collect();

      let backend_section = tera.render("section.html.tera", &Section {
        name: "Backend",
        cols: backend_html
          .chunks((backend_html.len() as f32 / 3.0).ceil() as usize)
          .collect(),
      }).unwrap();

      final_html.push_str(&backend_section);

      // write file

      let path = self._path.as_ref()
        .and_then(|x| x.parent())
        .and_then(|x| x.parent())
        .unwrap()
        .join("webserver")
        .join("web")
        .join("templates")
        .join("generated_credits.html.tera");
      let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
      file.write_all(final_html.as_bytes()).unwrap();
    }
  }

  crate fn credits() {
    let credits = Credits::load();
    credits.check_all_deps();
    credits.create_html();
  }
}

crate mod manifest {
  use serde_derive::Deserialize;

  use std::collections::BTreeMap;

  #[derive(Debug, Deserialize)]
  crate struct Manifest {
    crate dependencies: BTreeMap<String, toml::Value>,
    crate package: Package,
  }

  #[derive(Debug, Deserialize)]
  crate struct Package {
    crate metadata: Metadata,
  }

  #[derive(Debug, Deserialize)]
  crate struct Metadata {
    crate credits: Credits,
  }

  #[derive(Debug, Deserialize)]
  crate struct Credits {
    crate ignore: Vec<String>,
  }
}
