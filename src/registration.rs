use gazpatcho::config::NodeTemplate;
use std::boxed::Box;
use std::collections::HashMap;

pub use graphity::Node;

pub trait ModuleClass<N, C, P>: Send + Sync {
    fn instantiate(&self, data: HashMap<String, gazpatcho::model::Value>) -> Box<dyn Module<N>>;
    fn template(&self) -> NodeTemplate;
    fn consumer(&self, class: &str) -> C;
    fn producer(&self, class: &str) -> P;
}

pub trait Module<N> {
    fn take_node(&mut self) -> N;
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {}
}
