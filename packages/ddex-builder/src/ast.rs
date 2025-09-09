//! Abstract Syntax Tree for DDEX XML generation

use indexmap::IndexMap;
// Remove unused serde imports since we're not serializing AST

/// Abstract Syntax Tree root
#[derive(Debug, Clone)]
pub struct AST {
    pub root: Element,
    pub namespaces: IndexMap<String, String>,
    pub schema_location: Option<String>,
}

/// XML Element
#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: IndexMap<String, String>,
    pub children: Vec<Node>,
}

/// XML Node types
#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

impl Element {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: None,
            attributes: IndexMap::new(),
            children: Vec::new(),
        }
    }
    
    pub fn with_namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }
    
    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
    
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.children.push(Node::Text(text.into()));
        self
    }
    
    pub fn add_child(&mut self, child: Element) {
        self.children.push(Node::Element(child));
    }
    
    pub fn add_text(&mut self, text: impl Into<String>) {
        self.children.push(Node::Text(text.into()));
    }
}