use fluent_bundle::{FluentBundle, FluentResource, FluentValue};

use tera::{
  Error as TeraError,
  GlobalFn,
  Value as TeraValue,
};

use unic_langid::parser::parse_language_identifier;

use std::{
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
}

pub fn bundles() -> Result<Vec<FluentBundle<FluentResource>>, I18nError> {
  let mut bundles = Vec::new();
  for entry in Path::new("./i18n/").read_dir().map_err(I18nError::Io)? {
    let entry = entry.map_err(I18nError::Io)?.path();
    let lang_id = match entry.file_stem().and_then(OsStr::to_str) {
      Some(s) => s,
      None => continue,
    };
    let ftl = std::fs::read_to_string(&entry).map_err(I18nError::Io)?;
    let resource = FluentResource::try_new(ftl).map_err(|(_, e)| I18nError::FluentParse(e))?;
    let lang_id = parse_language_identifier(&lang_id).map_err(I18nError::LangId)?;
    let mut bundle = FluentBundle::new(&[lang_id.clone()]);
    bundle.add_resource(resource).map_err(I18nError::Fluents)?;
    bundles.push(bundle);
  }
  Ok(bundles)
}

pub fn tera_function(bundles: Vec<FluentBundle<FluentResource>>) -> GlobalFn {
  Box::new(move |mut args: HashMap<String, TeraValue>| {
    let langs = args.remove("_langs")
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

    let langs: Vec<_> = langs
      .into_iter()
      .flat_map(|x| parse_language_identifier(&x))
      .collect();

    let default_bundle = bundles.iter()
      .find(|b| b.locales.contains(&unic_langid::langid!("en")))
      .ok_or_else(|| TeraError::from("missing english translations"))?;
    let bundle = langs.iter()
      .flat_map(|l| bundles.iter().find(|b| b.locales.contains(l)))
      .next()
      .or_else(|| Some(&default_bundle))
      .ok_or_else(|| TeraError::from("missing translations"))?;

    let mut fell_back = false;
    let message = bundle.get_message(&msg)
      .or_else(|| {
        fell_back = true;
        default_bundle.get_message(&msg)
      })
      .ok_or_else(|| TeraError::from(format!("missing message {} in translation {}", msg, bundle.locales[0])))?;
    let pattern = match attr {
      Some(attr) => message.attributes.get(attr.as_str())
        .map(std::ops::Deref::deref)
        .ok_or_else(|| TeraError::from(format!("message {} did not have a pattern for attribute {}", msg, attr))),
      None => message.value
        .ok_or_else(|| TeraError::from(format!("message {} did not have a pattern", msg))),
    }?;

    let args = args
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
    let mut errors = Vec::new();
    let bundle = if fell_back { default_bundle } else { bundle };
    let output = bundle.format_pattern(
      &pattern,
      Some(&args),
      &mut errors,
    );
    Ok(TeraValue::from(output))
  })

}
