use crate::config::Config;

use html5ever::{
  parse_fragment, serialize, QualName, Parser,
  driver::ParseOpts,
  rcdom::{NodeData, RcDom, Handle},
  tendril::TendrilSink,
  tree_builder::TreeSink,
  interface::Attribute,
};

use crypto::{
  hmac::Hmac,
  mac::Mac,
  sha1::Sha1,
};

use url::{Url, ParseError as UrlParseError};

fn make_parser() -> Parser<RcDom> {
  parse_fragment(
    RcDom::default(),
    ParseOpts::default(),
    QualName::new(None, ns!(html), local_name!("div")),
    vec![],
  )
}

fn walk(config: &Config, handle: Handle, external: &Attribute) {
  let node = handle;
  match node.data {
    NodeData::Element { ref name, ref attrs, .. } if name.local == local_name!("img") => {
      let mut new_url = match crate::CAMO_URL.as_ref() {
        Some(u) => u.clone(),
        None => return,
      };
      let mut attrs = attrs.borrow_mut();
      let mut url_attr = match attrs.iter_mut().find(|x| x.name.local == local_name!("src")) {
        Some(a) => a,
        None => return,
      };

      let url = match Url::parse(&url_attr.value) {
        Ok(u) => u,
        Err(_) => return,
      };

      let mut hmac = Hmac::new(Sha1::new(), &crate::CAMO_KEY);
      hmac.input(url.as_str().as_bytes());
      let hmac_encoded = hex::encode(&hmac.result().code());

      // FIXME: unwrap
      new_url
        .path_segments_mut()
        .unwrap()
        .pop_if_empty()
        .push(&hmac_encoded);
      new_url
        .query_pairs_mut()
        .append_pair("url", url.as_str());

      url_attr.value = new_url.into_string().into();
    },
    NodeData::Element { ref name, ref attrs, .. } if name.local == local_name!("a") => {
      let url = attrs
        .borrow()
        .iter()
        .find(|x| x.name.local == local_name!("href"))
        .map(|x| Url::parse(&x.value));
      match url {
        // mark the url as external if it doesn't point to our host
        Some(Ok(ref u)) if u.host_str().is_some() && u.host_str() != Some(&config.general.site_domain) => {
          attrs.borrow_mut().push(external.clone());
        },
        // do not mark relative urls
        Some(Err(UrlParseError::RelativeUrlWithoutBase)) => {},
        // mark the url as external if url parsing failed
        Some(Err(_)) => {
          attrs.borrow_mut().push(external.clone());
        },
        // do not mark other urls
        _ => {},
      }
    },
    _ => {},
  }

  for child in node.children.borrow().iter() {
    walk(config, child.clone(), external);
  }
}

pub fn process(config: &Config, src: &str) -> String {
  let external = Attribute {
    name: QualName::new(None, ns!(), local_name!("class")),
    value: "external".into(),
  };

  let parser = make_parser();

  let mut dom = parser.one(src);

  walk(config, dom.get_document(), &external);

  let mut s = Vec::default();
  serialize(&mut s, &dom.document.children.borrow()[0], Default::default()).expect("serialization failed");

  String::from_utf8_lossy(&s).to_string()
}
