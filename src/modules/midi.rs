use gazpatcho::config as c;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Drop;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::samples::{self, Samples};

pub struct Class {
    input_devices: HashMap<String, portmidi::DeviceInfo>,
}

impl Class {
    pub fn new() -> Self {
        let context = portmidi::PortMidi::new().unwrap();
        let input_devices = context
            .devices()
            .unwrap()
            .into_iter()
            .filter(|d| d.is_input())
            .map(|d| (format!("in{}", d.id()), d))
            .collect();
        Self { input_devices }
    }
}

impl<N, C, P> crate::registration::Module<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(
        &self,
        _id: String,
        _data: HashMap<String, gazpatcho::model::Value>,
    ) -> (Box<dyn crate::Widget>, N) {
        let mut daemon = Rc::new(RefCell::new(Daemon::default()));
        let input_devices = self.input_devices.clone();
        if !input_devices.is_empty() {
            daemon
                .borrow_mut()
                .set_device(input_devices.values().next().unwrap().clone());
        }
        (
            Box::new(Module {
                input_devices: input_devices,
                daemon: Rc::clone(&daemon),
            }),
            Node::new(daemon).into(),
        )
    }

    fn template(&self) -> c::NodeTemplate {
        c::NodeTemplate {
            label: "MIDI".to_owned(),
            class: "midi".to_owned(),
            display_heading: true,
            pins: vec![
                c::Pin {
                    label: "Freq".to_owned(),
                    class: "freq".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "Gate".to_owned(),
                    class: "gate".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "Velocity".to_owned(),
                    class: "velocity".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "Pitchbend".to_owned(),
                    class: "pitchbend".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC1".to_owned(),
                    class: "cc1".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC2".to_owned(),
                    class: "cc2".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC3".to_owned(),
                    class: "cc3".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC4".to_owned(),
                    class: "cc4".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC5".to_owned(),
                    class: "cc5".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC6".to_owned(),
                    class: "cc6".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC7".to_owned(),
                    class: "cc7".to_owned(),
                    direction: c::Output,
                },
                c::Pin {
                    label: "CC8".to_owned(),
                    class: "cc8".to_owned(),
                    direction: c::Output,
                },
            ],
            widgets: vec![c::DropDown {
                key: "input".to_owned(),
                items: self
                    .input_devices
                    .iter()
                    .map(|(k, d)| c::DropDownItem {
                        label: d.name().to_string(),
                        value: k.to_string(),
                    })
                    .collect(),
            }],
        }
    }

    fn consumer(&self, _class: &str) -> C {
        unreachable!();
    }

    fn producer(&self, class: &str) -> P {
        match class {
            "freq" => Producer::Frequency.into(),
            "gate" => Producer::Gate.into(),
            "velocity" => Producer::Velocity.into(),
            "pitchbend" => Producer::Pitchbend.into(),
            "cc1" => Producer::CC1.into(),
            "cc2" => Producer::CC2.into(),
            "cc3" => Producer::CC3.into(),
            "cc4" => Producer::CC4.into(),
            "cc5" => Producer::CC5.into(),
            "cc6" => Producer::CC6.into(),
            "cc7" => Producer::CC7.into(),
            "cc8" => Producer::CC8.into(),
            _ => unreachable!(),
        }
    }
}

pub struct Module {
    daemon: Rc<RefCell<Daemon>>,
    input_devices: HashMap<String, portmidi::DeviceInfo>,
}

impl crate::registration::Widget for Module {
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {
        let input_key = data.get("input").unwrap().unwrap_string();
        self.daemon
            .borrow_mut()
            .set_device(self.input_devices.get(input_key).unwrap().clone());
    }
}

#[derive(Default)]
pub struct Node {
    daemon: Rc<RefCell<Daemon>>,
    active: Option<wmidi::Note>,
    freq: Samples,
    gate: Samples,
    velocity: Samples,
    pitchbend: Samples,
    cc1: Samples,
    cc2: Samples,
    cc3: Samples,
    cc4: Samples,
    cc5: Samples,
    cc6: Samples,
    cc7: Samples,
    cc8: Samples,
}

impl Node {
    fn new(daemon: Rc<RefCell<Daemon>>) -> Self {
        Node {
            daemon,
            ..Self::default()
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Producer {
    Frequency,
    Gate,
    Velocity,
    Pitchbend,
    CC1,
    CC2,
    CC3,
    CC4,
    CC5,
    CC6,
    CC7,
    CC8,
}

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn read(&self, producer: Producer) -> Samples {
        match producer {
            Producer::Frequency => self.freq,
            Producer::Gate => self.gate,
            Producer::Velocity => self.velocity,
            Producer::Pitchbend => self.pitchbend,
            Producer::CC1 => self.cc1,
            Producer::CC2 => self.cc2,
            Producer::CC3 => self.cc3,
            Producer::CC4 => self.cc4,
            Producer::CC5 => self.cc5,
            Producer::CC6 => self.cc6,
            Producer::CC7 => self.cc7,
            Producer::CC8 => self.cc8,
        }
    }

    fn tick(&mut self) {
        for message in self.daemon.borrow().try_iter() {
            let message = wmidi::MidiMessage::try_from(&message.message[..]).unwrap();
            match message {
                wmidi::MidiMessage::NoteOn(_, note, _) => {
                    self.active = Some(note);
                    self.freq = samples::value(note.to_freq_f32());
                    self.gate = samples::value(1.0);
                }
                wmidi::MidiMessage::NoteOff(_, note, _) => {
                    if let Some(active) = self.active {
                        if active == note {
                            self.gate = samples::value(0.0);
                        }
                    }
                }
                wmidi::MidiMessage::ControlChange(_, function, value) => {
                    let value: f32 = u8::from(value).into();
                    let value = samples::value(value / 128.0);
                    match u8::from(function) {
                        1 => self.cc1 = value,
                        2 => self.cc2 = value,
                        3 => self.cc3 = value,
                        4 => self.cc4 = value,
                        5 => self.cc5 = value,
                        6 => self.cc6 = value,
                        7 => self.cc7 = value,
                        8 => self.cc8 = value,
                        _ => (),
                    }
                }
                wmidi::MidiMessage::PitchBendChange(_, pitchbend) => {
                    let mut pitchbend: f32 = u16::from(pitchbend).into();
                    pitchbend -= f32::powi(2.0, 13);
                    pitchbend /= f32::powi(2.0, 13);
                    self.pitchbend = samples::value(pitchbend);
                }
                _ => (),
            }
        }
    }
}

struct Message {
    message: [u8; 4],
}

struct TryIter<'a> {
    message_rx: Option<&'a mpsc::Receiver<Message>>,
}

impl<'a> Iterator for TryIter<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        match self.message_rx {
            Some(message_rx) => message_rx.try_recv().ok(),
            None => None,
        }
    }
}

#[derive(Default)]
struct Daemon {
    join_handle: Option<thread::JoinHandle<()>>,
    message_rx: Option<mpsc::Receiver<Message>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl Daemon {
    fn set_device(&mut self, device_info: portmidi::DeviceInfo) {
        if let Some(stop_tx) = &self.stop_tx.take() {
            stop_tx.send(()).unwrap();
        }

        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        }

        let (message_tx, message_rx) = mpsc::channel();
        self.message_rx = Some(message_rx);

        let (stop_tx, stop_rx) = mpsc::channel();
        self.stop_tx = Some(stop_tx);

        let join_handle = thread::spawn(move || {
            let timeout = Duration::from_millis(1000 / 128);

            let context = portmidi::PortMidi::new().unwrap();
            let input_port = context.input_port(device_info, 8).unwrap();

            while input_port.poll().is_ok() {
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                if let Ok(Some(events)) = input_port.read_n(8) {
                    for event in events {
                        message_tx
                            .send(Message {
                                message: [
                                    event.message.status,
                                    event.message.data1,
                                    event.message.data2,
                                    event.message.data3,
                                ],
                            })
                            .unwrap();
                    }
                }

                thread::sleep(timeout);
            }
        });

        self.join_handle = Some(join_handle);
    }

    fn try_iter(&self) -> TryIter {
        TryIter {
            message_rx: self.message_rx.as_ref(),
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Some(stop_tx) = &self.stop_tx {
            stop_tx.send(()).unwrap();
            self.join_handle.take().unwrap().join().unwrap();
        }
    }
}
