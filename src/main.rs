// TODO: Add PWM, waveform and sync
// TODO:
// - adsr
// - math1, math4, math8
// - midi channels
// - midi polyphony
// - reverb
// - delay
// - vco based on wavetable
// - different waveforms in vco
// - sample and hold
// - quantizer
// - filter
// - clock with divider
// - slew
// - wavefolder
// - frequency analyzer
// - fix drop down on MIDI

#![allow(clippy::large_enum_variant)]

#[macro_use]
extern crate graphity;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod action;
mod bank;
mod diff;
mod modules;
mod registration;
mod samples;
mod stream;

use cpal::traits::StreamTrait;
use gazpatcho::model::PinAddress;
use graphity::{NodeIndex, NodeWrapper};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use crate::action::Action;
use crate::modules::dac;
use crate::modules::math;
use crate::modules::midi;
use crate::modules::scope;
use crate::modules::value;
use crate::modules::vco;
use crate::registration::{Module, Widget};
use crate::samples::Samples;

const SAMPLE_RATE: u32 = 48000;

graphity!(
    Graph<Samples>;
    Bank = {bank::Node, bank::Consumer, bank::Producer},
    Math = {math::Node, math::Consumer, math::Producer},
    Value = {value::Node, value::Consumer, value::Producer},
    Scope = {scope::Node, scope::Consumer, scope::Producer},
    VCO = {vco::Node, vco::Consumer, vco::Producer},
    MIDI = {midi::Node, midi::Consumer, midi::Producer},
    DAC = {dac::Node, dac::Consumer, dac::Producer},
);

lazy_static! {
    static ref CLASSES: HashMap<String, Box<dyn Module<__Node, __Consumer, __Producer>>> = {
        let classes: Vec<Box<dyn Module<__Node, __Consumer, __Producer>>> = vec![
            Box::new(value::Value),
            Box::new(scope::Scope),
            Box::new(math::Math),
            Box::new(vco::VCO),
            Box::new(dac::DAC),
            Box::new(midi::MIDI::new()),
        ];
        classes
            .into_iter()
            .map(|c| (c.template().class, c))
            .collect()
    };
}

const SINK_NODE_CLASS: &str = "dac";
const SINK_NODE_OUTPUT_PIN: &str = "out";

pub fn main() {
    let (ui_report_tx, ui_report_rx) = mpsc::channel();
    let (ui_request_tx, ui_request_rx) = mpsc::channel();
    let (output_stream, data_req_rx, data_tx) = stream::build_output_stream(SAMPLE_RATE);
    let (ui_action_tx, ui_action_rx) = mpsc::channel();

    run_ui_handler(ui_report_rx, ui_request_tx.clone(), ui_action_tx);

    run_graph_handler(data_req_rx, ui_request_tx, ui_action_rx, data_tx);

    output_stream
        .play()
        .expect("Failed playing the output stream");

    let config = gazpatcho::config::Config {
        node_templates: CLASSES.iter().map(|(_, c)| c.template()).collect(),
    };
    gazpatcho::run_with_mpsc("Sirena", config, ui_report_tx, ui_request_rx);
}

fn run_ui_handler(
    ui_report_rx: mpsc::Receiver<gazpatcho::report::Report>,
    ui_request_tx: mpsc::Sender<gazpatcho::request::Request>,
    ui_action_tx: mpsc::Sender<Action>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sink_node_instance: Option<String> = None;

        let mut old_report: Option<gazpatcho::report::Report> = None;

        for report in ui_report_rx {
            let diff = diff::Diff::new(old_report.as_ref(), &report);

            old_report = Some(report);

            for patch in diff.removed_patches {
                ui_action_tx.send(Action::RemovePatch(patch)).unwrap();
            }

            for node in diff.removed_nodes {
                if let Some(sink_node_instance_id) = &sink_node_instance {
                    if node.id == *sink_node_instance_id {
                        sink_node_instance = None;
                    }
                }

                ui_action_tx.send(Action::RemoveNode(node)).unwrap();
            }

            for node in diff.added_nodes {
                let add_output_action = if node.class == SINK_NODE_CLASS {
                    if sink_node_instance.is_some() {
                        ui_request_tx
                            .send(gazpatcho::request::Request::RemoveNode {
                                node_id: node.id.clone(),
                            })
                            .unwrap();
                        None
                    } else {
                        sink_node_instance = Some(node.id.clone());
                        Some(Action::AddOutputPatch(gazpatcho::model::PinAddress {
                            node_id: node.id.clone(),
                            pin_class: SINK_NODE_OUTPUT_PIN.to_owned(),
                        }))
                    }
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
    })
}

fn run_graph_handler(
    data_req_rx: mpsc::Receiver<()>,
    ui_request_tx: mpsc::Sender<gazpatcho::request::Request>,
    ui_action_rx: mpsc::Receiver<Action>,
    data_tx: mpsc::Sender<Samples>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        struct Meta {
            pub widget: Option<Box<dyn Widget>>,
            pub node_index: __NodeIndex,
            pub class: String,
        }
        let mut meta: HashMap<String, Meta> = HashMap::new();

        let mut graph = Graph::new();

        let output = graph.add_node(bank::Node::new());

        let get_producer_index = |meta: &HashMap<String, Meta>, pin_address: &PinAddress| {
            let source = meta.get(&pin_address.node_id).unwrap();
            let producer = CLASSES
                .get(&source.class)
                .unwrap()
                .producer(&pin_address.pin_class);
            source.node_index.producer(producer)
        };

        let get_consumer_index = |meta: &HashMap<String, Meta>, pin_address: &PinAddress| {
            let destination = meta.get(&pin_address.node_id).unwrap();
            let consumer = CLASSES
                .get(&destination.class)
                .unwrap()
                .consumer(&pin_address.pin_class);
            destination.node_index.consumer(consumer)
        };

        for _ in data_req_rx {
            graph.tick();
            let data = graph.node(&output).unwrap().read(bank::Producer);
            data_tx.send(data).unwrap();

            for action in ui_action_rx.try_iter() {
                match action {
                    Action::AddNode(node) => {
                        let instance = CLASSES
                            .get(&node.class)
                            .unwrap()
                            .instantiate(node.id.clone());
                        let (mut widget, node_) = (instance.widget, instance.node);
                        if let Some(widget) = &mut widget {
                            widget.update(node.data);
                            widget.register_ui_tx(ui_request_tx.clone());
                        }
                        let node_index = graph.add_node(node_);
                        meta.insert(
                            node.id,
                            Meta {
                                widget,
                                node_index,
                                class: node.class,
                            },
                        );
                    }
                    Action::UpdateNode(node) => {
                        if let Some(widget) = &mut meta.get_mut(&node.id).unwrap().widget {
                            widget.update(node.data);
                        }
                    }
                    Action::RemoveNode(node) => {
                        let node_index = meta.get(&node.id).unwrap().node_index;
                        graph.remove_node(node_index);
                        meta.remove(&node.id);
                    }
                    Action::AddPatch(patch) => {
                        let producer_index = get_producer_index(&meta, &patch.source);
                        let consumer_index = get_consumer_index(&meta, &patch.destination);
                        graph.must_add_edge(producer_index, consumer_index);
                    }
                    Action::RemovePatch(patch) => {
                        let producer_index = get_producer_index(&meta, &patch.source);
                        let consumer_index = get_consumer_index(&meta, &patch.destination);
                        graph.remove_edge(producer_index, consumer_index);
                    }
                    Action::AddOutputPatch(pin_address) => {
                        let producer_index = get_producer_index(&meta, &pin_address);
                        graph.must_add_edge(producer_index, output.consumer(bank::Consumer));
                    }
                }
            }
        }
    })
}
