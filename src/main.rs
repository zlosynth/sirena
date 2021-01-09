#[macro_use]
extern crate graphity;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod action;
mod diff;
mod modules;
mod registration;
mod stream;

use cpal::traits::StreamTrait;
use graphity::{NodeIndex, NodeWrapper};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use crate::action::Action;
use crate::modules::bank;
use crate::modules::dac;
use crate::modules::math;
use crate::modules::vco;
use crate::registration::{Module, ModuleClass};

const SAMPLE_RATE: u32 = 44800;

graphity!(
    Graph<[f32; 32]>;
    Bank = {bank::Bank, bank::Input, bank::Output},
    DAC = {dac::Node, dac::Consumer, dac::Producer},
    VCO = {vco::Node, vco::Consumer, vco::Producer},
    Math = {math::Node, math::Consumer, math::Producer},
);

lazy_static! {
    static ref CLASSES: HashMap<String, Box<dyn ModuleClass<__Node, __Consumer, __Producer>>> = {
        let classes: Vec<Box<dyn ModuleClass<__Node, __Consumer, __Producer>>> = vec![
            Box::new(math::Class),
            Box::new(vco::Class),
            Box::new(dac::Class),
        ];
        classes
            .into_iter()
            .map(|c| (c.template().class, c))
            .collect()
    };
}

pub fn main() {
    let config = gazpatcho::config::Config {
        node_templates: CLASSES.iter().map(|(_, c)| c.template()).collect(),
    };

    let (ui_report_tx, ui_report_rx) = mpsc::channel::<gazpatcho::report::Report>();
    let (ui_request_tx, ui_request_rx) = mpsc::channel();

    let (output_stream, data_req_rx, data_tx) = stream::build_output_stream(SAMPLE_RATE);

    let (ui_action_tx, ui_action_rx) = mpsc::channel();

    thread::spawn(move || {
        // TODO: Run UI handler, compares reports, reverts unwanted, passes events to the graph thread
        // TODO: Make sure there is one DAC at most
        // TODO: Custom module for diff, translating the diff into list of actions

        let mut old_report: Option<gazpatcho::report::Report> = None;

        for report in ui_report_rx {
            let diff = diff::Diff::new(old_report.as_ref(), &report);

            old_report = Some(report);

            for patch in diff.removed_patches {
                ui_action_tx.send(Action::RemovePatch(patch)).unwrap();
            }

            for node in diff.removed_nodes {
                ui_action_tx.send(Action::RemoveNode(node)).unwrap();
            }

            for node in diff.added_nodes {
                let add_output_action = if node.class == "dac" {
                    Some(Action::AddOutputPatch(gazpatcho::model::PinAddress {
                        node_id: node.id.clone(),
                        pin_class: "out".to_owned(),
                    }))
                } else {
                    None
                };
                ui_action_tx.send(Action::AddNode(node)).unwrap();
                if let Some(add_output_action) = add_output_action {
                    ui_action_tx.send(add_output_action).unwrap();
                }
            }

            for node in diff.updated_nodes {
                ui_action_tx.send(Action::UpdateNode(node)).unwrap();
            }

            for patch in diff.added_patches {
                ui_action_tx.send(Action::AddPatch(patch)).unwrap();
            }
        }
    });

    // TODO: Split fetching of data and reacting to actions
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

            for action in ui_action_rx.try_iter() {
                match action {
                    Action::AddNode(node) => {
                        let mut module = CLASSES.get(&node.class).unwrap().instantiate(node.data);
                        let node_index = graph.add_node(module.take_node());
                        nodes.insert(node.id.clone(), node_index);
                        class_by_module.insert(node.id.clone(), node.class);
                        modules.insert(node.id, module);
                    }
                    Action::UpdateNode(node) => {
                        modules.get_mut(&node.id).unwrap().update(node.data);
                    }
                    Action::RemoveNode(node) => {
                        let node_index = nodes.get(&node.id).unwrap();
                        graph.remove_node(*node_index);
                        nodes.remove(&node.id);
                        class_by_module.remove(&node.id);
                        modules.remove(&node.id);
                    }
                    Action::AddPatch(patch) => {
                        let source_node_class = class_by_module.get(&patch.source.node_id).unwrap();
                        let source_node_index = nodes.get(&patch.source.node_id).unwrap();
                        let producer = CLASSES
                            .get(source_node_class)
                            .unwrap()
                            .producer(&patch.source.pin_class);
                        let producer_index = source_node_index.producer(producer);
                        let destination_node_class =
                            class_by_module.get(&patch.destination.node_id).unwrap();
                        let destination_node_index = nodes.get(&patch.destination.node_id).unwrap();
                        let consumer = CLASSES
                            .get(destination_node_class)
                            .unwrap()
                            .consumer(&patch.destination.pin_class);
                        let consumer_index = destination_node_index.consumer(consumer);
                        graph.must_add_edge(producer_index, consumer_index);
                    }
                    Action::RemovePatch(patch) => {
                        let source_node_class = class_by_module.get(&patch.source.node_id).unwrap();
                        let source_node_index = nodes.get(&patch.source.node_id).unwrap();
                        let producer = CLASSES
                            .get(source_node_class)
                            .unwrap()
                            .producer(&patch.source.pin_class);
                        let producer_index = source_node_index.producer(producer);
                        let destination_node_class =
                            class_by_module.get(&patch.destination.node_id).unwrap();
                        let destination_node_index = nodes.get(&patch.destination.node_id).unwrap();
                        let consumer = CLASSES
                            .get(destination_node_class)
                            .unwrap()
                            .consumer(&patch.destination.pin_class);
                        let consumer_index = destination_node_index.consumer(consumer);
                        graph.remove_edge(producer_index, consumer_index);
                    }
                    Action::AddOutputPatch(pin_address) => {
                        let source_node_class = class_by_module.get(&pin_address.node_id).unwrap();
                        let source_node_index = nodes.get(&pin_address.node_id).unwrap();
                        let producer = CLASSES
                            .get(source_node_class)
                            .unwrap()
                            .producer(&pin_address.pin_class);
                        let producer_index = source_node_index.producer(producer);
                        graph.must_add_edge(producer_index, output.consumer(bank::Input));
                    }
                }
            }
        }
    });

    output_stream
        .play()
        .expect("Failed playing the output stream");

    gazpatcho::run_with_mpsc("Sirena", config, ui_report_tx, ui_request_rx);
}
