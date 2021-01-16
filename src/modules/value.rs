use gazpatcho::config as c;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::samples::{self, Samples};

pub struct Class;

impl<N, C, P> crate::registration::Module<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, id: String) -> (Box<dyn crate::Widget>, N) {
        let value = Arc::new(Mutex::new(0.0));
        (
            Box::new(Module {
                id,
                value: Arc::clone(&value),
                join_handle: None,
                stop_tx: None,
            }),
            Node {
                value: value,
                ..Node::default()
            }
            .into(),
        )
    }

    fn template(&self) -> c::NodeTemplate {
        c::NodeTemplate {
            label: "Value".to_owned(),
            class: "value".to_owned(),
            display_heading: false,
            pins: vec![
                c::Pin {
                    label: "In".to_owned(),
                    class: "in".to_owned(),
                    direction: c::Input,
                },
                c::Pin {
                    label: "Out".to_owned(),
                    class: "out".to_owned(),
                    direction: c::Output,
                },
            ],
            widgets: vec![c::TextBox {
                key: "value".to_owned(),
                capacity: 100,
                size: [80.0, 20.0],
                read_only: false,
            }],
        }
    }

    fn consumer(&self, _class: &str) -> C {
        Consumer.into()
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

pub struct Module {
    id: String,
    value: Arc<Mutex<f32>>,
    stop_tx: Option<mpsc::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl crate::registration::Widget for Module {
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {
        let value = data.get("value").unwrap().unwrap_string();
        let value = if value.is_empty() {
            Ok(0.0)
        } else {
            value.parse::<f32>()
        };
        if let Ok(value) = value {
            *self.value.lock().unwrap() = value;
        }
    }

    fn register_ui_tx(&mut self, ui_tx: mpsc::Sender<gazpatcho::request::Request>) {
        let (stop_tx, stop_rx) = mpsc::channel();

        let value = Arc::clone(&self.value);
        let id = self.id.clone();

        let join_handle = thread::spawn(move || loop {
            ui_tx
                .send(gazpatcho::request::Request::SetValue {
                    node_id: id.clone(),
                    key: "value".to_string(),
                    value: gazpatcho::model::Value::String((*value.lock().unwrap()).to_string()),
                })
                .unwrap();

            if stop_rx.try_recv().is_ok() {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        });

        self.stop_tx = Some(stop_tx);
        self.join_handle = Some(join_handle);
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        if let Some(stop_tx) = &self.stop_tx {
            stop_tx.send(()).unwrap();
            self.join_handle.take().unwrap().join().unwrap();
        }
    }
}

#[derive(Default)]
pub struct Node {
    value: Arc<Mutex<f32>>,
    values: Samples,
    written_to: bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: Samples) {
        self.written_to = true;
        self.values = data;
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.values
    }

    fn tick(&mut self) {
        if self.written_to {
            *self.value.lock().unwrap() = self.values[0];
            self.written_to = false;
        } else {
            self.values = samples::value(*self.value.lock().unwrap());
        }
    }
}
