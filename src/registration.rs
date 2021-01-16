// TODO: Have 3 modules, one exposing all from graphity, one from gazpatcho, one for module

use gazpatcho::config::NodeTemplate;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::mpsc;

pub use graphity::Node;

pub trait Module<N, C, P>: Send + Sync {
    fn instantiate(
        &self,
        id: String,
        data: HashMap<String, gazpatcho::model::Value>,
    ) -> (Box<dyn Widget>, N);
    fn template(&self) -> NodeTemplate;
    fn consumer(&self, class: &str) -> C;
    fn producer(&self, class: &str) -> P;
}

// TODO: Rename to handle. Class would return node and handle
pub trait Widget {
    #[allow(unused_variables)]
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {}
    #[allow(unused_variables)]
    fn register_ui_tx(&mut self, ui_tx: mpsc::Sender<gazpatcho::request::Request>) {}
}
