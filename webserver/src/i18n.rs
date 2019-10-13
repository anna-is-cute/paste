use fluent_bundle::{FluentBundle, FluentResource, FluentValue};

use serde_derive::Deserialize;

use tera::{
  Error as TeraError,
  GlobalFn,
  Value as TeraValue,
};

use unic_langid::{
  parser::parse_language_identifier,
  LanguageIdentifier,
};

use std::{
  borrow::Cow,
  collections::HashMap,
  ffi::OsStr,
  path::Path,
};

#[derive(Debug)]
pub enum I18nError {
  Io(std::io::Error),
  Fluents(Vec<fluent_bundle::FluentError>),
  FluentParse(Vec<fluent_syntax::parser::errors::ParserError>),
  LangId(unic_langid::parser::errors::ParserError),
  Toml(toml::de::Error),
}

fn manifest() -> Result<Manifest, I18nError> {
  let f = std::fs::read_to_string("./i18n/manifest.toml").map_err(I18nError::Io)?;
  toml::from_str(&f).map_err(I18nError::Toml)
}

fn bundles() -> Result<HashMap<LanguageIdentifier, FluentBundle<FluentResource>>, I18nError> {
  let mut bundles = HashMap::new();
  for entry in Path::new("./i18n/").read_dir().map_err(I18nError::Io)? {
    let entry = entry.map_err(I18nError::Io)?.path();
    if entry.extension() != Some(std::ffi::OsStr::new("ftl")) {
      continue;
    }
    let lang_id = match entry.file_stem().and_then(OsStr::to_str) {
      Some(s) => s,
      None => continue,
    };
    let ftl = std::fs::read_to_string(&entry).map_err(I18nError::Io)?;
    let resource = FluentResource::try_new(ftl).map_err(|(_, e)| I18nError::FluentParse(e))?;
    let lang_id = parse_language_identifier(&lang_id).map_err(I18nError::LangId)?;
    let mut bundle = FluentBundle::new(&[lang_id.clone()]);
    bundle.add_resource(resource).map_err(I18nError::Fluents)?;
    bundles.insert(lang_id, bundle);
  }
  Ok(bundles)
}

type Bundle = FluentBundle<FluentResource>;

#[derive(Deserialize)]
struct Manifest {
  #[serde(flatten, deserialize_with = "self::langid::deserialize_map")]
  bundles: HashMap<LanguageIdentifier, ManifestBundle>,
}

#[derive(Deserialize)]
struct ManifestBundle {
  #[serde(deserialize_with = "self::langid::deserialize_vec")]
  serves: Vec<LanguageIdentifier>,
  #[serde(default, deserialize_with = "self::langid::deserialize_vec")]
  fallbacks: Vec<LanguageIdentifier>,
}

pub struct Localisation {
  manifest: Manifest,
  bundles: HashMap<LanguageIdentifier, Bundle>,
}

impl Localisation {
  pub fn new() -> Result<Self, I18nError> {
    let manifest = manifest()?;
    let bundles = bundles()?;
    Ok(Localisation {
      manifest,
      bundles,
    })
  }

  fn bundle(&self, want: &LanguageIdentifier) -> Option<(&ManifestBundle, &Bundle)> {
    for (lang, bundle) in &self.manifest.bundles {
      if bundle.serves.iter().any(|l| l == want) {
        return self.bundles.get(lang).map(|b| (bundle, b));
      }
    }

    None
  }

  fn bundles(&self, wants: &[LanguageIdentifier]) -> Vec<&Bundle> {
    let mut bundles = Vec::new();
    for want in wants {
      if let Some((mb, bundle)) = self.bundle(want) {
        bundles.push(bundle);
        bundles.append(&mut self.bundles(&mb.fallbacks));
      }
    }
    bundles
  }

  fn message<'a, 'b: 'a>(&'b self, req: MessageRequest<'a>) -> Result<Cow<'a, str>, TeraError> {
    let found = self.bundles(req.wants)
      .into_iter()
      .flat_map(|bundle| {
        let mut message = bundle.get_message(req.msg)?;
        let pattern = match req.attr {
          Some(attr) => message.attributes.remove(attr),
          None => message.value,
        }?;
        Some((bundle, pattern))
      })
      .next();

    let (bundle, pattern) = found
      .ok_or_else(|| TeraError::from(format!(
        "could not find message {} in any of these locales: {}",
        match req.attr {
          Some(attr) => format!("{} with attribute {}", req.msg, attr),
          None => req.msg.to_string(),
        },
        req.wants.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "),
      )))?;

    let mut errors = Vec::new();
    let output = bundle.format_pattern(
      pattern,
      req.args,
      &mut errors,
    );
    for error in errors {
      eprintln!("error while getting localised message: {}", error);
    }
    Ok(output)
  }
}

struct MessageRequest<'a> {
  wants: &'a [LanguageIdentifier],
  msg: &'a str,
  attr: Option<&'a str>,
  args: Option<&'a HashMap<&'a str, FluentValue<'a>>>,
}

pub fn tera_function(localisation: Localisation) -> GlobalFn {
  Box::new(move |mut args: HashMap<String, TeraValue>| {
    let langs: Vec<LanguageIdentifier> = args.remove("_langs")
      .and_then(|v| match v {
        TeraValue::String(s) => Some(vec![s]),
        TeraValue::Array(vals) => {
          if !vals.iter().all(|x| x.is_string()) {
            return None;
          }
          Some(vals.into_iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect())
        },
        _ => None,
      })
      .map(|vs| vs.into_iter()
        .flat_map(|v| parse_language_identifier(&v))
        .collect())
      .ok_or_else(|| TeraError::from("missing _lang parameter"))?;
    let msg = args.remove("_msg")
      .and_then(|v| match v {
        TeraValue::String(s) => Some(s),
        _ => None,
      })
      .ok_or_else(|| TeraError::from("missing _msg parameter"))?;
    let attr = args.remove("_attr")
      .and_then(|v| match v {
        TeraValue::String(s) => Some(s),
        _ => None,
      });
    let args: HashMap<&str, FluentValue> = args
      .iter()
      .map(|(key, value)| {
        let value = match value {
          TeraValue::String(ref s) => FluentValue::String(s.into()),
          TeraValue::Number(ref n) => FluentValue::Number(n.to_string().into()),
          _ => return Err(TeraError::from("translation args must be strings or numbers")),
        };
        Ok((key.as_str(), value))
      })
      .collect::<Result<_, _>>()?;

    let output = localisation.message(MessageRequest {
      wants: &langs,
      msg: &msg,
      attr: attr.as_ref().map(AsRef::as_ref),
      args: if args.is_empty() { None } else { Some(&args) },
    })?;

    Ok(TeraValue::from(output))
  })

}

mod langid {
  use serde::{
    Deserialize,
    de::{self, Deserializer},
  };

  use unic_langid::{
    LanguageIdentifier,
    parser::parse_language_identifier,
  };

  use std::collections::HashMap;

  #[derive(Deserialize, Hash, PartialEq, Eq)]
  struct LangId(#[serde(deserialize_with = "deserialize")] LanguageIdentifier);

  pub fn deserialize<'de, D>(des: D) -> Result<LanguageIdentifier, D::Error>
    where D: Deserializer<'de>,
  {
    let s = String::deserialize(des)?;
    parse_language_identifier(s.as_str()).map_err(de::Error::custom)
  }

  pub fn deserialize_vec<'de, D>(des: D) -> Result<Vec<LanguageIdentifier>, D::Error>
    where D: Deserializer<'de>,
  {
    let v: Vec<LangId> = Vec::deserialize(des)?;
    Ok(v.into_iter().map(|LangId(id)| id).collect())
  }

  pub fn deserialize_map<'de, D, V>(des: D) -> Result<HashMap<LanguageIdentifier, V>, D::Error>
    where D: serde::de::Deserializer<'de>,
          V: serde::Deserialize<'de>,
  {
    let v: HashMap<LangId, V> = HashMap::deserialize(des)?;
    Ok(v.into_iter().map(|(LangId(id), v)| (id, v)).collect())
  }
}
