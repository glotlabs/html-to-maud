use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Render {
    Auto,
    Full,
    OnlyBody,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IdStyle {
    Full,
    Short,
    ShortNoDiv,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ClassStyle {
    Full,
    Short,
    ShortNoDiv,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub render: Render,
    pub id_style: IdStyle,
    pub class_style: ClassStyle,
}

struct Doc {
    input: String,
    head: Vec<String>,
    body: Vec<String>,
}

impl Doc {
    fn new(input: &str) -> Doc {
        Doc {
            input: input.to_string(),
            head: vec![],
            body: vec![],
        }
    }

    fn render(&self, config: &Config) -> String {
        match config.render {
            Render::Auto => {
                let root_elems = vec!["<html>", "<head>", "<body>"];
                let render_only_body = !root_elems.iter().any(|elem| self.input.contains(elem));

                if render_only_body {
                    self.render_body()
                } else {
                    self.render_full()
                }
            }
            Render::Full => self.render_full(),
            Render::OnlyBody => self.render_body(),
        }
    }

    #[rustfmt::skip]
    fn render_full(&self) -> String {
        let head_str = self.indent_vec(&self.head, 8).join("\n");
        let body_str = self.indent_vec(&self.body, 8).join("\n");

        vec![
            "html! {",
            "    (maud::DOCTYPE)",
            "    head {",
            &head_str,
            "    }",
            "    body {",
            &body_str,
            "    }",
            "}",
        ].join("\n")
    }

    #[rustfmt::skip]
    fn render_body(&self) -> String {
        let body_str = self.indent_vec(&self.body, 4).join("\n");

        vec![
            "html! {",
            &body_str,
            "}",
        ].join("\n")
    }

    fn push(&mut self, parent: &Parent, content: String) {
        match parent {
            Parent::Head => self.head.push(content),
            Parent::Body => self.body.push(content),
            Parent::Other => (),
        }
    }

    fn indent_vec(&self, v: &[String], indent: usize) -> Vec<String> {
        v.iter()
            .map(|s| format!("{:indent$}{}", "", s, indent = indent))
            .collect()
    }
}

#[derive(PartialEq, Eq)]
enum Parent {
    Head,
    Body,
    Other,
}

pub fn html_to_maud(html: &str, config: &Config) -> String {
    let mut input = html.as_bytes();
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut input)
        .unwrap(); // TODO: handle error

    let mut doc = Doc::new(html);
    walk(config, 0, &dom.document, &mut doc, &Parent::Other);

    doc.render(config)
}

fn walk(config: &Config, indent: usize, node: &Handle, doc: &mut Doc, parent: &Parent) {
    match &node.data {
        NodeData::Document => {
            for child in node.children.borrow().iter() {
                walk(config, indent + 4, child, doc, parent);
            }
        }

        NodeData::Doctype { .. } => {}

        NodeData::Text { contents } => {
            let text = &contents.borrow();
            let text = text.trim();
            if !text.is_empty() {
                let output = format!(
                    "{:indent$}\"{}\"",
                    "",
                    text.escape_default(),
                    indent = indent
                );

                match parent {
                    Parent::Head => doc.head.push(output),
                    Parent::Body => doc.body.push(output),
                    Parent::Other => (),
                }
            }
        }

        NodeData::Comment { .. } => {
            //println!("{:indent$}/* {} */", "", contents, indent = indent)
        }

        NodeData::Element { name, attrs, .. } => {
            let attributes = attrs.borrow().iter().map(new_attribute).collect::<Vec<_>>();
            let tag_name = name.local.to_string();
            let elem = Element::new(tag_name.clone(), attributes);

            let children = node.children.borrow();

            let curly_or_semicolon = if is_empty_element(&tag_name) {
                ";".to_string()
            } else {
                " {".to_string()
            };

            let output = format!(
                "{:indent$}{}{}",
                "",
                elem.to_maud(config),
                curly_or_semicolon,
                indent = indent
            );
            doc.push(parent, output);

            let new_parent = match tag_name.as_str() {
                "head" => &Parent::Head,
                "body" => &Parent::Body,
                _ => parent,
            };

            let new_indent = if tag_name == "head" || tag_name == "body" {
                0
            } else {
                indent + 4
            };

            for child in children.iter() {
                walk(config, new_indent, child, doc, new_parent);
            }

            if !is_empty_element(&tag_name) {
                let output = format!("{:indent$}}}", "", indent = indent);
                doc.push(parent, output);
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }
}

#[rustfmt::skip]
fn is_empty_element(tag_name: &str) -> bool {
    let void_tags = vec![
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr",
    ];

    void_tags.into_iter().any(|s| tag_name == s)
}

#[derive(Debug)]
struct Element {
    tag_name: String,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<(String, String)>,
}

impl Element {
    pub fn new(tag_name: String, attrs: Vec<Attribute>) -> Element {
        let info = Element {
            tag_name: tag_name,
            ids: Vec::new(),
            classes: Vec::new(),
            attributes: Vec::new(),
        };

        attrs.iter().fold(info, |mut info, attr| {
            match attr {
                Attribute::Id(id) => {
                    info.ids.push(id.to_string());
                }

                Attribute::Classes(classes) => {
                    info.classes
                        .extend(classes.split_whitespace().map(|s| s.to_string()));
                }

                Attribute::Other { name, value } => {
                    info.attributes.push((name.to_string(), value.to_string()));
                }
            }

            info
        })
    }

    pub fn to_maud(&self, config: &Config) -> String {
        vec![
            self.format_tag_name(config),
            self.format_id(&config.id_style),
            self.format_classes(&config.class_style),
            self.format_attributes(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(" ")
    }

    fn format_tag_name(&self, config: &Config) -> String {
        if self.should_omit_tag_name(config) {
            "".to_string()
        } else {
            self.tag_name.to_string()
        }
    }

    fn format_id(&self, id_style: &IdStyle) -> String {
        self.ids
            .first()
            .map(|id| self.format_id_helper(id_style, id))
            .unwrap_or_else(|| "".to_string())
    }

    fn format_id_helper(&self, id_style: &IdStyle, id: &str) -> String {
        match id_style {
            IdStyle::Full => format!(r#"id="{}""#, id),
            IdStyle::Short | IdStyle::ShortNoDiv => format!("#{}", self.shorthand_quote(id)),
        }
    }

    fn format_classes(&self, class_style: &ClassStyle) -> String {
        match class_style {
            ClassStyle::Full => {
                let classes = self.classes.join(" ");

                if classes.is_empty() {
                    "".to_string()
                } else {
                    format!(r#"class="{}""#, classes)
                }
            }

            ClassStyle::Short | ClassStyle::ShortNoDiv => {
                let classes = self
                    .classes
                    .iter()
                    .map(|class| self.shorthand_quote(class))
                    .collect::<Vec<_>>()
                    .join(".");

                if classes.is_empty() {
                    "".to_string()
                } else {
                    format!(".{}", classes)
                }
            }
        }
    }

    fn format_attributes(&self) -> String {
        self.attributes
            .iter()
            .map(|(name, value)| self.format_attribute(name, value))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn format_attribute(&self, name: &str, value: &str) -> String {
        if value.is_empty() {
            name.to_string()
        } else {
            format!("{}=\"{}\"", name, value)
        }
    }

    fn should_omit_tag_name(&self, config: &Config) -> bool {
        if self.tag_name != "div" {
            false
        } else if !self.ids.is_empty() && config.id_style == IdStyle::ShortNoDiv {
            true
        } else if !self.classes.is_empty() && config.class_style == ClassStyle::ShortNoDiv {
            true
        } else {
            false
        }
    }

    fn shorthand_quote(&self, str: &str) -> String {
        if self.shorthand_attr_required_quotes(str) {
            format!("\"{}\"", str)
        } else {
            str.to_string()
        }
    }

    fn shorthand_attr_required_quotes(&self, str: &str) -> bool {
        str.chars().any(char::is_numeric) || str.chars().any(|c| c == ':')
    }
}

enum Attribute {
    Id(String),
    Classes(String),
    Other { name: String, value: String },
}

fn new_attribute(attr: &html5ever::Attribute) -> Attribute {
    match &attr.name.local[..] {
        "id" => Attribute::Id(attr.value.to_string()),

        "class" => Attribute::Classes(attr.value.to_string()),

        _ => Attribute::Other {
            name: attr.name.local.to_string(),
            value: attr.value.to_string(),
        },
    }
}
