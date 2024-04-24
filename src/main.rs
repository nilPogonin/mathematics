use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

#[derive(Debug, Serialize, Deserialize)]
struct Tree {
    root: Option<Box<Node>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    value: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(value: i32) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

impl From<Node> for Option<Box<Node>> {
    fn from(node: Node) -> Self {
        Some(Box::new(node))
    }
}

impl Tree {
    fn new() -> Self {
        Tree { root: None }
    }

    fn insert(&mut self, value: i32) {
        match &mut self.root {
            None => {
                self.root = Node::new(value).into();
            }
            Some(node) => {
                Tree::insert_recursive(node, value);
            }
        }
    }

    fn insert_recursive(node: &mut Box<Node>, value: i32) {
        let child_node = if value > node.value {
            &mut node.right
        } else {
            &mut node.left
        };
        match child_node {
            None => *child_node = Node::new(value).into(),
            Some(n) => Tree::insert_recursive(n, value),
        }
    }

    fn to_json(&self) -> serde_json::Value {
        if let Some(root) = &self.root {
            return serde_json::json!(Tree::serialize_node(root));
        }
        serde_json::Value::Null
    }

    fn serialize_node(node: &Box<Node>) -> serde_json::Value {
        let mut children = Vec::new();
        if let Some(left) = &node.left {
            children.push(Tree::serialize_node(left));
        }
        if let Some(right) = &node.right {
            children.push(Tree::serialize_node(right));
        }
        serde_json::json!({
            "value": node.value,
            "children": children,
        })
    }
}

static TREE_INSTANCE: Lazy<Arc<Mutex<Tree>>> = Lazy::new(|| {
    let tree = Arc::new(Mutex::new(Tree::new()));
    tree
});

async fn add_node(node: web::Json<Node>) -> impl Responder {
    let value = node.value;
    let mut tree = TREE_INSTANCE.lock().unwrap();
    tree.insert(value);
    let tree_data = tree.to_json();
    HttpResponse::Ok().json(tree_data)
}

async fn index() -> impl Responder {
    match fs::read_to_string("src/index.html") {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::NotFound().body("Index file not found"),
    }
}

async fn tree_data() -> impl Responder {
    let tree_data = TREE_INSTANCE.lock().unwrap().to_json();
    HttpResponse::Ok().json(tree_data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/tree-data", web::get().to(tree_data))
            .route("/add-node", web::post().to(add_node))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}