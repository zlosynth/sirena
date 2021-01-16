// TODO: Have 3 modules, one exposing all from graphity, one from gazpatcho, one for module

use gazpatcho::config::NodeTemplate;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::mpsc;

pub use graphity::Node;

pub trait Module<N, C, P>: Send + Sync {
    fn instantiate(&self, id: String) -> crate::registration::ModuleInstance<N>;
    fn template(&self) -> NodeTemplate;
    fn consumer(&self, class: &str) -> C;
    fn producer(&self, class: &str) -> P;
}

pub struct ModuleInstance<N> {
    pub node: N,
    pub widget: Option<Box<dyn Widget>>,
}

impl<N> ModuleInstance<N> {
    pub fn new(node: N) -> Self {
        Self { node, widget: None }
    }

    pub fn with_widget(mut self, widget: Box<dyn Widget>) -> Self {
        self.widget = Some(widget);
        self
    }
}

// TODO: Rename to handle. Class would return node and handle
pub trait Widget {
    #[allow(unused_variables)]
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {}
    #[allow(unused_variables)]
    fn register_ui_tx(&mut self, ui_tx: mpsc::Sender<gazpatcho::request::Request>) {}
}
