// TODO Implement a buffer that would be used for input, continuously fed from
// input devices

// TODO: Keep a clock that would continously populate all input buffers

// XXX Use buffers for input (even from widgets) too, that way everything can
// behanlded RT

// XXX Rule, keep minimal functionality, if it can be done using a combination,
// don't duplicate. This should not build instruments, just provide building
// blocks.
//
// TODO: The maths module should understand nodes. C4 ...
// TODO: Buffer should be an option, to ignore ports that are not connected
// TODO: Keep it straight to the point of the current needs, no stress until v1

#[macro_use]
extern crate graphity;

#[cfg(test)]
#[macro_use]
extern crate approx;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use graphity::{NodeIndex, NodeWrapper};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

mod modules;

use modules::bank;
use modules::dac;
use modules::math;
use modules::vco;

graphity!(
    Instrument<[f32; 32]>;
    Bank = {bank::Bank, bank::Input, bank::Output},
    DAC = {dac::DAC, dac::Input, dac::Output},
    Math = {math::Math, math::Input, math::Output},
    VCO = {vco::VCO, vco::Input, vco::Output},
);

// With 512 buffer, the resolution on input widgets would be 0.01. Should be ok
// for most usecase. Plus we should use smaller buffers anyway.

// 1. Values can be loaded into graph using shared pointer (laptop) / refcell (laptop)
// 2. It can be either handled in a separate module (portable, but expensive) or in the user directly
// Can we make the direct usage portable?

struct RingBuffer {
    buffer: Option<[f32; 32]>,
    index: usize,
    req_tx: mpsc::Sender<()>,
    data_rx: mpsc::Receiver<[f32; 32]>,
}

impl RingBuffer {
    pub fn new() -> (Self, mpsc::Receiver<()>, mpsc::Sender<[f32; 32]>) {
        let (req_tx, req_rx) = mpsc::channel();
        let (data_tx, data_rx) = mpsc::channel();
        req_tx.send(()).unwrap();
        let buffer = Self {
            buffer: None,
            index: 0,
            req_tx,
            data_rx,
        };
        thread::sleep(std::time::Duration::from_millis(1000));
        (buffer, req_rx, data_tx)
    }

    pub fn pop(&mut self) -> f32 {
        if self.index == 0 {
            let new_data = self.data_rx.recv().unwrap(); // TODO panic if fell behind
            self.req_tx.send(()).unwrap();
            self.buffer = Some(new_data);
        }

        let value = self.buffer.unwrap()[self.index];

        if self.index == 31 {
            self.index = 0;
        } else {
            self.index += 1;
        }

        value
    }
}

fn new_gazpatcho_config() -> gazpatcho::config::Config {
    use gazpatcho::config::*;

    let oscillator = vco::template();
    let dac = dac::template();
    let math = math::template();

    let config = Config {
        node_templates: vec![dac, oscillator, math],
    };

    config
}

fn main() {
    let (report_tx, report_rx) = mpsc::channel::<gazpatcho::report::Report>();
    let (request_tx, request_rx) = mpsc::channel::<gazpatcho::request::Request>();

    let (mut buffer, data_req_rx, data_tx) = { RingBuffer::new() };

    let mut instrument = Instrument::new();

    thread::spawn(move || {
        let mut instrument = Instrument::new();

        let output = instrument.add_node(bank::Bank::default());

        // let frequency_generator = instrument.add_node(generator::Generator::new(440.0));
        // let main_oscillator = instrument.add_node(oscillator::Oscillator::default());
        // let output_sink = instrument.add_node(sink::Sink::default());

        // instrument.must_add_edge(
        //     frequency_generator.producer(generator::Output),
        //     main_oscillator.consumer(oscillator::Input),
        // );
        // instrument.must_add_edge(
        //     main_oscillator.producer(oscillator::Output),
        //     output_sink.consumer(sink::Input),
        // );

        fn diff_report(
            old: &gazpatcho::report::Report,
            new: &gazpatcho::report::Report,
        ) -> (
            Vec<gazpatcho::model::Node>,
            Vec<gazpatcho::model::Node>,
            Vec<String>,
            Vec<gazpatcho::model::Patch>,
            Vec<gazpatcho::model::Patch>,
        ) {
            let old_by_id: std::collections::HashMap<_, _> = old
                .nodes
                .iter()
                .map(|n| (n.id.clone(), n.clone()))
                .collect();
            let new_by_id: std::collections::HashMap<_, _> = new
                .nodes
                .iter()
                .map(|n| (n.id.clone(), n.clone()))
                .collect();

            let mut new_nodes = Vec::new();
            let mut updated_nodes = Vec::new();

            for (k, n) in new_by_id.iter() {
                if !old_by_id.contains_key(k) {
                    new_nodes.push(n.clone());
                } else if old_by_id[k] != *n {
                    updated_nodes.push(n.clone());
                }
            }

            let removed_ids = old_by_id
                .keys()
                .filter(|k| !new_by_id.contains_key(*k))
                .cloned()
                .collect();

            let old_patches: HashSet<_> = old.patches.iter().collect();
            let new_patches: HashSet<_> = new.patches.iter().collect();

            let added_patches = new_patches
                .difference(&old_patches)
                .cloned()
                .cloned()
                .collect();
            let removed_patches = old_patches
                .difference(&new_patches)
                .cloned()
                .cloned()
                .collect();

            (
                new_nodes,
                updated_nodes,
                removed_ids,
                added_patches,
                removed_patches,
            )
        }

        let mut rep = None;

        let mut index_by_id = std::collections::HashMap::new();
        let mut class_by_id = std::collections::HashMap::new();
        let mut formula_by_id = std::collections::HashMap::new();

        for _ in data_req_rx {
            instrument.tick();
            let data = instrument.node(&output).unwrap().read(bank::Output);
            data_tx.send(data).unwrap();

            for report in report_rx.try_iter() {
                let (new_nodes, updated_nodes, removed_nodes, new_patches, removed_patches) =
                    if let Some(old_report) = rep {
                        diff_report(&old_report, &report)
                    } else {
                        let new_nodes = report.nodes.iter().cloned().collect();
                        let new_patches = report.patches.iter().cloned().collect();
                        (new_nodes, vec![], vec![], new_patches, vec![])
                    };

                for n in &new_nodes {
                    match n.class.as_str() {
                        "math" => {
                            println!("Added {:?}", n.id);
                            let formula = n.data.get("formula").unwrap().unwrap_string();
                            let formula = if let Ok(formula) = formula.parse() {
                                formula
                            } else {
                                "0".parse().unwrap()
                            };
                            let formula = Rc::new(RefCell::new(formula));
                            formula_by_id.insert(n.id.clone(), Rc::clone(&formula));
                            let index = instrument.add_node(math::Math::new(formula));
                            index_by_id.insert(n.id.clone(), index);
                            class_by_id.insert(n.id.clone(), "math");
                        }
                        "vco" => {
                            println!("Added {:?}", n.id);
                            let index = instrument.add_node(vco::VCO::default());
                            index_by_id.insert(n.id.clone(), index);
                            class_by_id.insert(n.id.clone(), "vco");
                        }
                        "dac" => {
                            println!("Added {:?}", n.id);
                            let index = instrument.add_node(dac::DAC::default());
                            instrument.add_edge(
                                index.producer(dac::Output),
                                output.consumer(bank::Input),
                            );
                            index_by_id.insert(n.id.clone(), index);
                            class_by_id.insert(n.id.clone(), "dac");
                        }
                        &_ => unreachable!(),
                    }
                }

                for n in &updated_nodes {
                    match n.class.as_str() {
                        "math" => {
                            let formula = n.data.get("formula").unwrap().unwrap_string();
                            let formula = std::panic::catch_unwind(|| formula.parse());
                            if let Ok(formula) = formula {
                                if let Ok(formula) = formula {
                                    *formula_by_id.get(&n.id).unwrap().borrow_mut() = formula;
                                }
                            } else {
                                println!("BAD ERROR: parse panics");
                            }
                        }
                        "vco" => {
                            println!("Updated {:?}", n.id);
                        }
                        "dac" => {
                            println!("Updated {:?}", n.id);
                        }
                        &_ => unreachable!(),
                    }
                }

                for n in &removed_nodes {
                    index_by_id.remove(n);
                    formula_by_id.remove(n);
                }

                for e in &new_patches {
                    let source = index_by_id.get(&e.source.node_id).unwrap();
                    let destination = index_by_id.get(&e.destination.node_id).unwrap();

                    let producer = {
                        match *class_by_id.get(&e.source.node_id).unwrap() {
                            "math" => match e.source.pin_class.as_str() {
                                "out" => source.producer(math::Output),
                                &_ => unreachable!(),
                            },
                            "vco" => match e.source.pin_class.as_str() {
                                "out" => source.producer(vco::Output),
                                &_ => unreachable!(),
                            },
                            "dac" => unreachable!(),
                            &_ => unreachable!(),
                        }
                    };

                    let consumer = {
                        match *class_by_id.get(&e.destination.node_id).unwrap() {
                            "math" => match e.destination.pin_class.as_str() {
                                "x" => destination.consumer(math::Input::In1),
                                "y" => destination.consumer(math::Input::In2),
                                &_ => unreachable!(),
                            },
                            "vco" => match e.destination.pin_class.as_str() {
                                "freq" => destination.consumer(vco::Input::Frequency),
                                &_ => unreachable!(),
                            },
                            "dac" => match e.destination.pin_class.as_str() {
                                "in" => destination.consumer(dac::Input),
                                &_ => unreachable!(),
                            },
                            &_ => unreachable!(),
                        }
                    };

                    instrument.add_edge(producer, consumer);
                }

                for e in &removed_patches {
                    let source = index_by_id.get(&e.source.node_id).unwrap();
                    let destination = index_by_id.get(&e.destination.node_id).unwrap();

                    let producer = {
                        match *class_by_id.get(&e.source.node_id).unwrap() {
                            "math" => match e.source.pin_class.as_str() {
                                "out" => source.producer(math::Output),
                                &_ => unreachable!(),
                            },
                            "vco" => match e.source.pin_class.as_str() {
                                "out" => source.producer(vco::Output),
                                &_ => unreachable!(),
                            },
                            "dac" => unreachable!(),
                            &_ => unreachable!(),
                        }
                    };

                    let consumer = {
                        match *class_by_id.get(&e.destination.node_id).unwrap() {
                            "math" => match e.destination.pin_class.as_str() {
                                "x" => destination.consumer(math::Input::In1),
                                "y" => destination.consumer(math::Input::In2),
                                &_ => unreachable!(),
                            },
                            "vco" => match e.destination.pin_class.as_str() {
                                "freq" => destination.consumer(vco::Input::Frequency),
                                &_ => unreachable!(),
                            },
                            "dac" => match e.destination.pin_class.as_str() {
                                "in" => destination.consumer(dac::Input),
                                &_ => unreachable!(),
                            },
                            &_ => unreachable!(),
                        }
                    };

                    instrument.remove_edge(producer, consumer);
                }

                dbg!(index_by_id.keys());
                dbg!(new_nodes);
                dbg!(updated_nodes);
                dbg!(removed_nodes);

                rep = Some(report);
            }
        }
    });

    let stream = build_stream(buffer);
    stream.play().unwrap();

    let config = new_gazpatcho_config();
    gazpatcho::run_with_mpsc("Sirena", config, report_tx, request_rx);
}

fn build_stream(mut buffer: RingBuffer) -> cpal::Stream {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    let supported_configs = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs
        .filter(|c| c.sample_format() == cpal::SampleFormat::F32 && c.channels() == 2)
        .next()
        .unwrap()
        .with_sample_rate(cpal::SampleRate(44800));
    let config: cpal::StreamConfig = supported_config.into();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    // TODO: Once stream goes out of scope, it stops playing
    device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(2) {
                    let value = cpal::Sample::from::<f32>(&buffer.pop());
                    frame[0] = value;
                    frame[1] = value;
                }
            },
            err_fn,
        )
        .unwrap()
}
