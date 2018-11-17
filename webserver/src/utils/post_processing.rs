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

#[derive(Default)]
struct Context {
  first_in_li: bool,
}

fn walk(config: &Config, handle: Handle, external: &Attribute, ctx: &mut Context) -> bool {
  let node = handle;

  match node.data {
    NodeData::Element { ref name, .. } if &*name.local == "li" => ctx.first_in_li = true,
    NodeData::Text { ref contents } if contents.borrow().trim() == "" => {},
    NodeData::Element { ref name, ref attrs, .. } if &*name.local == "input" => {
      let was_first = ctx.first_in_li;
      ctx.first_in_li = false;

      if !attrs.borrow().iter().any(|x| &*x.name.local == "type") {
        return false;
      }

      let parent = node.parent.take();
      if let Some(node) = parent.as_ref().and_then(|x| x.upgrade()) {
        match node.data {
          NodeData::Element { ref name, .. } if &*name.local != "li" => return false,
          _ => {},
        }
      }
      node.parent.replace(parent);

      if !was_first {
        return false;
      }
    },
    _ => ctx.first_in_li = false,
  }

  match node.data {
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

  node
    .children
    .borrow_mut()
    .retain(|child| walk(config, child.clone(), external, ctx));

  true
}

pub fn process(config: &Config, src: &str) -> String {
  let external = Attribute {
    name: QualName::new(None, ns!(), local_name!("class")),
    value: "external".into(),
  };

  let parser = make_parser();

  let mut dom = parser.one(src);

  let mut ctx = Context::default();

  walk(config, dom.get_document(), &external, &mut ctx);

  let mut s = Vec::default();
  serialize(&mut s, &dom.document.children.borrow()[0], Default::default()).expect("serialization failed");

  String::from_utf8_lossy(&s).to_string()
}
