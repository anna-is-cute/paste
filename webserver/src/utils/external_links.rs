use crate::config::Config;

use html5ever::{
  parse_fragment, serialize, QualName, Parser,
  driver::ParseOpts,
  rcdom::{NodeData, RcDom, Handle},
  tendril::TendrilSink,
  tree_builder::TreeSink,
  interface::Attribute,
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

pub fn mark(config: &Config, src: &str) -> String {
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
