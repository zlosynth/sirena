// TODO: Kepe this as PoC lab, don't try too hard to mka eit into a lib
// add instrument.rs, with struct, implement modules over it

// TODO: Turn this into a library (limit to UI)
// TODO: Define:
// - MIDI input (start mono, with note and gate)
// - ADSR
// - filter
// TODO: Buffer should be an option, to ignore ports that are not connected

#[macro_use]
extern crate graphity;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod modules;
mod stream;

// TODO
// each module will be boxed and kept in a hash map
// the module would keep all its metadata (refcells)
// the module would reconcile actions of gazpatcho (the node type would be used as a key in hashmap)

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use gazpatcho::config::{Config, NodeTemplate};
use gazpatcho::report::Report;
use gazpatcho::request::Request;
use graphity::{NodeIndex, NodeWrapper};
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use crate::modules::bank;
use crate::modules::dac;
use crate::modules::math;
use crate::modules::vco;

const SAMPLE_RATE: u32 = 44800;

graphity!(
    Graph<[f32; 32]>;
    Bank = {bank::Bank, bank::Input, bank::Output},
    DAC = {dac::DAC, dac::Input, dac::Output},
    VCO = {vco::Node, vco::Consumer, vco::Producer},
    Math = {math::Math, math::Input, math::Output},
);

trait ModuleClass<N, C, P>: Send {
    fn instantiate(&self) -> Box<dyn Module<N>>;
    fn template(&self) -> NodeTemplate;
    fn consumer(&self, class: &str) -> C;
    fn producer(&self, class: &str) -> P;
}

trait Module<N> {
    fn take_node(&mut self) -> N;
}

struct DACClass;

impl<N, C, P> ModuleClass<N, C, P> for DACClass
where
    N: From<dac::DAC>,
    C: From<dac::Input>,
    P: From<dac::Output>,
{
    fn instantiate(&self) -> Box<dyn Module<N>> {
        Box::new(DAC)
    }

    fn template(&self) -> NodeTemplate {
        dac::template()
    }

    fn consumer(&self, class: &str) -> C {
        dac::Input.into()
    }

    fn producer(&self, class: &str) -> P {
        dac::Output.into()
    }
}

struct DAC;

impl<N> Module<N> for DAC
where
    N: From<dac::DAC>,
{
    fn take_node(&mut self) -> N {
        dac::DAC::default().into()
    }
}

struct MathClass;

impl<N, C, P> ModuleClass<N, C, P> for MathClass
where
    N: From<math::Math>,
    C: From<math::Input>,
    P: From<math::Output>,
{
    fn instantiate(&self) -> Box<dyn Module<N>> {
        let formula = Rc::new(RefCell::new("440".parse().unwrap()));
        Box::new(Math { formula })
    }

    fn template(&self) -> NodeTemplate {
        math::template()
    }

    // TODO: Implement From<&str> trait on Input
    fn consumer(&self, class: &str) -> C {
        match class {
            "x" => math::Input::In1.into(),
            "y" => math::Input::In2.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, class: &str) -> P {
        math::Output.into()
    }
}

struct Math {
    formula: Rc<RefCell<meval::Expr>>,
}

impl<N> Module<N> for Math
where
    N: From<math::Math>,
{
    fn take_node(&mut self) -> N {
        math::Math::new(Rc::clone(&self.formula)).into()
    }
}

// TODO: Maybe just pass gazpatcho report structs inside
enum Action {
    AddNode {
        class: String,
        id: String,
    },
    UpdateNode,
    RemoveNode,
    AddOutputPatch {
        source_node_id: String,
        source_pin_class: String,
    },
    AddPatch {
        source_node_id: String,
        source_pin_class: String,
        destination_node_id: String,
        destination_pin_class: String,
    },
    RemovePatch,
}

pub fn main() {
    let classes: Vec<Box<dyn ModuleClass<__Node, __Consumer, __Producer>>> = vec![
        Box::new(MathClass),
        Box::new(vco::Class),
        Box::new(DACClass),
    ];

    let classes: HashMap<_, Box<dyn ModuleClass<__Node, __Consumer, __Producer>>> = classes
        .into_iter()
        .map(|c| (c.template().class, c))
        .collect();

    let (output_stream, data_req_rx, data_tx) = stream::build_output_stream(SAMPLE_RATE);

    let (ui_action_tx, ui_action_rx) = mpsc::channel::<Action>();

    thread::spawn(move || {
        // TODO: Run UI handler, compares reports, reverts unwanted, passes events to the graph thread
        // TODO: Make sure there is one DAC at most

        ui_action_tx
            .send(Action::AddNode {
                class: "vco".to_owned(),
                id: "vco:1".to_owned(),
            })
            .unwrap();
        ui_action_tx
            .send(Action::AddNode {
                class: "dac".to_owned(),
                id: "dac:1".to_owned(),
            })
            .unwrap();
        ui_action_tx
            .send(Action::AddOutputPatch {
                source_node_id: "dac:1".to_owned(),
                source_pin_class: "out".to_owned(),
            })
            .unwrap();
        ui_action_tx
            .send(Action::AddNode {
                class: "math".to_owned(),
                id: "math:1".to_owned(),
            })
            .unwrap();

        ui_action_tx.send(Action::AddPatch {
            source_node_id: "math:1".to_owned(),
            source_pin_class: "out".to_owned(),
            destination_node_id: "vco:1".to_owned(),
            destination_pin_class: "freq".to_owned(),
        });
        ui_action_tx.send(Action::AddPatch {
            source_node_id: "vco:1".to_owned(),
            source_pin_class: "out".to_owned(),
            destination_node_id: "dac:1".to_owned(),
            destination_pin_class: "in".to_owned(),
        });
    });

    thread::spawn(move || {
        // TODO: single map with right hand struct for module, node and class
        let mut modules = HashMap::new();
        let mut nodes = HashMap::new();
        let mut class_by_module = HashMap::new();

        let mut graph = Graph::new();
        let output = graph.add_node(bank::Bank::default());

        for _ in data_req_rx {
            graph.tick();
            let data = graph.node(&output).unwrap().read(bank::Output);
            data_tx.send(data).unwrap();

            for action in &ui_action_rx {
                match action {
                    Action::AddNode { class, id } => {
                        let mut module = classes.get(&class).unwrap().instantiate();
                        let node_index = graph.add_node(module.take_node());
                        nodes.insert(id.clone(), node_index);
                        class_by_module.insert(id.clone(), class);
                        modules.insert(id, module);
                    }
                    Action::AddPatch {
                        source_node_id,
                        source_pin_class,
                        destination_node_id,
                        destination_pin_class,
                    } => {
                        let source_node_class = class_by_module.get(&source_node_id).unwrap();
                        let source_node_index = nodes.get(&source_node_id).unwrap();
                        let producer = classes
                            .get(source_node_class)
                            .unwrap()
                            .producer(&source_pin_class);
                        let producer_index = source_node_index.producer(producer);
                        let destination_node_class =
                            class_by_module.get(&destination_node_id).unwrap();
                        let destination_node_index = nodes.get(&destination_node_id).unwrap();
                        let consumer = classes
                            .get(destination_node_class)
                            .unwrap()
                            .consumer(&destination_pin_class);
                        let consumer_index = destination_node_index.consumer(consumer);
                        graph.must_add_edge(producer_index, consumer_index);
                    }
                    Action::AddOutputPatch {
                        source_node_id,
                        source_pin_class,
                    } => {
                        let source_node_class = class_by_module.get(&source_node_id).unwrap();
                        let source_node_index = nodes.get(&source_node_id).unwrap();
                        let producer = classes
                            .get(source_node_class)
                            .unwrap()
                            .producer(&source_pin_class);
                        let producer_index = source_node_index.producer(producer);
                        graph.must_add_edge(producer_index, output.consumer(bank::Input));
                    }
                    _ => println!("unhandled action"),
                }
            }
        }
    });

    output_stream
        .play()
        .expect("Failed playing the output stream");

    loop {}

    // let config = Config {
    //     node_templates: classes.iter().map(|(k, c)| c.template()).collect(),
    // };
    // let (ui_report_tx, ui_report_rx) = mpsc::channel::<Report>();
    // let (ui_request_tx, ui_request_rx) = mpsc::channel::<Request>();
    // gazpatcho::run_with_mpsc("Sirena", config, ui_report_tx, ui_request_rx);
}

// fn diff_report(
//     old: &gazpatcho::report::Report,
//     new: &gazpatcho::report::Report,
// ) -> (
//     Vec<gazpatcho::model::Node>,
//     Vec<gazpatcho::model::Node>,
//     Vec<String>,
//     Vec<gazpatcho::model::Patch>,
//     Vec<gazpatcho::model::Patch>,
// ) {
//     let old_by_id: HashMap<_, _> = old
//         .nodes
//         .iter()
//         .map(|n| (n.id.clone(), n.clone()))
//         .collect();
//     let new_by_id: HashMap<_, _> = new
//         .nodes
//         .iter()
//         .map(|n| (n.id.clone(), n.clone()))
//         .collect();

//     let mut new_nodes = Vec::new();
//     let mut updated_nodes = Vec::new();

//     for (k, n) in new_by_id.iter() {
//         if !old_by_id.contains_key(k) {
//             new_nodes.push(n.clone());
//         } else if old_by_id[k] != *n {
//             updated_nodes.push(n.clone());
//         }
//     }

//     let removed_ids = old_by_id
//         .keys()
//         .filter(|k| !new_by_id.contains_key(*k))
//         .cloned()
//         .collect();

//     let old_patches: HashSet<_> = old.patches.iter().collect();
//     let new_patches: HashSet<_> = new.patches.iter().collect();

//     let added_patches = new_patches
//         .difference(&old_patches)
//         .cloned()
//         .cloned()
//         .collect();
//     let removed_patches = old_patches
//         .difference(&new_patches)
//         .cloned()
//         .cloned()
//         .collect();

//     (
//         new_nodes,
//         updated_nodes,
//         removed_ids,
//         added_patches,
//         removed_patches,
//     )
// }