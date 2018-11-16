use crate::config::Config;

use html5ever::{
  local_name, namespace_url, ns, parse_fragment, serialize, Parser, QualName,
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

fn walk(config: &Config, handle: Handle, external: &Attribute) -> bool {
  let node = handle;

  node
    .children
    .borrow_mut()
    .retain(|child| walk(config, child.clone(), external));

  match node.data {
    NodeData::Element { ref name, ref attrs, .. } if &*name.local == "input" => {
      let attrs = attrs.borrow();
      let type_attr = match attrs.iter().find(|x| &*x.name.local == "type") {
        Some(a) => a,
        None => return false,
      };

      if type_attr.value.trim().is_empty() {
        return false;
      }
    },
    NodeData::Element { ref name, ref attrs, .. } if &*name.local == "img" => {
      let mut new_url = match crate::CAMO_URL.as_ref() {
        Some(u) => u.clone(),
        None => return true,
      };
      let mut attrs = attrs.borrow_mut();
      let mut url_attr = match attrs.iter_mut().find(|x| &*x.name.local == "src") {
        Some(a) => a,
        None => return true,
      };

      let url = match Url::parse(&url_attr.value) {
        Ok(u) => u,
        Err(_) => return true,
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
    NodeData::Element { ref name, ref attrs, .. } if &*name.local == "a" => {
      let url = attrs
        .borrow()
        .iter()
        .find(|x| &*x.name.local == "href")
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

  true
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
