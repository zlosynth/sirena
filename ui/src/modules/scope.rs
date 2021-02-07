use gazpatcho::config::*;
use std::cmp::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::registration::{Module, ModuleInstance};
use crate::samples::{self, Samples};
use crate::SAMPLE_RATE;

const BUFFER_SIZE: usize = 2000;
const CANVAS_WIDTH: f32 = 400.0;
const CANVAS_HEIGHT: f32 = 200.0;

pub struct Scope;

impl<N, C, P> Module<N, C, P> for Scope
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, id: String) -> ModuleInstance<N> {
        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let node = Node::new(Arc::clone(&buffer)).into();
        let widget = Box::new(Widget {
            id,
            buffer,
            join_handle: None,
            stop_tx: None,
        });
        ModuleInstance::new(node).with_widget(widget)
    }

    fn template(&self) -> NodeTemplate {
        NodeTemplate {
            label: "Scope".to_owned(),
            class: "scope".to_owned(),
            display_heading: false,
            pins: vec![Pin {
                label: "In".to_owned(),
                class: "in".to_owned(),
                direction: Input,
            }],
            widgets: vec![Canvas {
                key: "scope".to_owned(),
                size: [CANVAS_WIDTH, CANVAS_HEIGHT],
            }],
        }
    }

    fn consumer(&self, _class: &str) -> C {
        Consumer.into()
    }

    fn producer(&self, _class: &str) -> P {
        unreachable!();
    }
}

pub struct Widget {
    id: String,
    buffer: Arc<Mutex<Buffer>>,
    stop_tx: Option<mpsc::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl crate::registration::Widget for Widget {
    fn register_ui_tx(&mut self, ui_tx: mpsc::Sender<gazpatcho::request::Request>) {
        let (stop_tx, stop_rx) = mpsc::channel();

        let id = self.id.clone();

        let mut prev_y = 0;

        let buffer = Arc::clone(&self.buffer);

        let join_handle = thread::spawn(move || loop {
            let mut wave = Vec::new();
            {
                let buffer = buffer.lock().unwrap();

                let max_delta = buffer
                    .buffer
                    .iter()
                    .fold(0.0, |max, x| f32::max(max, f32::abs(*x)));
                let scale = (CANVAS_HEIGHT / 2.0) / max_delta;

                for x in 0..CANVAS_WIDTH as usize {
                    let mut new_y = -buffer.buffer[x * buffer.occupied / CANVAS_WIDTH as usize];
                    new_y *= scale;
                    new_y += CANVAS_HEIGHT / 2.0;
                    let new_y = new_y as i32;

                    if x == 0 {
                        wave.push((x as f32, new_y as f32));
                    } else {
                        match new_y.cmp(&prev_y) {
                            Ordering::Greater => {
                                for y in prev_y + 1..=new_y {
                                    wave.push((x as f32, y as f32));
                                }
                            }
                            Ordering::Less => {
                                for y in new_y..prev_y {
                                    wave.push((x as f32, y as f32));
                                }
                            }
                            Ordering::Equal => wave.push((x as f32, new_y as f32)),
                        }
                    }

                    prev_y = new_y;
                }
            }
            ui_tx
                .send(gazpatcho::request::Request::SetValue {
                    node_id: id.clone(),
                    key: "scope".to_string(),
                    value: gazpatcho::model::Value::VecF32F32(wave),
                })
                .unwrap();

            if stop_rx.try_recv().is_ok() {
                break;
            }

            thread::sleep(Duration::from_millis(10));
        });

        self.stop_tx = Some(stop_tx);
        self.join_handle = Some(join_handle);
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        if let Some(stop_tx) = &self.stop_tx {
            stop_tx.send(()).unwrap();
            self.join_handle.take().unwrap().join().unwrap();
        }
    }
}

pub struct Node {
    input: Samples,
    buffer: Arc<Mutex<Buffer>>,
    buffer_index: usize,
    sum: f32,
    sum_n: u32,
    interval: u32,
    since_tick: u32,
    prev_val: f32,
    awaiting_reset: bool,
}

impl Node {
    fn new(buffer: Arc<Mutex<Buffer>>) -> Self {
        Self {
            buffer,
            buffer_index: 0,
            input: samples::zeroed(),
            sum: 0.0,
            sum_n: 0,
            interval: SAMPLE_RATE,
            since_tick: 0,
            prev_val: 0.0,
            awaiting_reset: true,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: Samples) {
        self.input = data;
    }

    fn tick(&mut self) {
        for i in self.input.iter() {
            self.since_tick += 1;

            if self.prev_val < 0.0 && *i > 0.0 {
                self.awaiting_reset = false;
                self.interval = self.since_tick;
                self.since_tick = 0;
            }

            if self.since_tick > SAMPLE_RATE {
                self.awaiting_reset = false;
                self.interval = SAMPLE_RATE;
            }

            self.prev_val = *i;

            if self.awaiting_reset {
                continue;
            }

            self.sum += *i;
            self.sum_n += 1;

            let zoom_in = u32::max(1, self.interval * 3 / BUFFER_SIZE as u32);
            self.buffer.lock().unwrap().occupied =
                usize::min(self.interval as usize * 3, BUFFER_SIZE);

            if self.sum_n >= zoom_in {
                {
                    let mut buffer = self.buffer.lock().unwrap();
                    buffer.buffer[self.buffer_index] = self.sum / self.sum_n as f32;
                }

                self.buffer_index += 1;
                if self.buffer_index == BUFFER_SIZE {
                    self.awaiting_reset = true;
                    self.buffer_index = 0;
                }

                self.sum = 0.0;
                self.sum_n = 0;
            }
        }
    }
}

struct Buffer {
    pub buffer: [f32; BUFFER_SIZE],
    pub occupied: usize,
}

impl Buffer {
    fn new() -> Self {
        let buffer = {
            let mut data: [std::mem::MaybeUninit<f32>; BUFFER_SIZE] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            for elem in &mut data[..] {
                unsafe {
                    std::ptr::write(elem.as_mut_ptr(), 0.0);
                }
            }
            unsafe { std::mem::transmute::<_, [f32; BUFFER_SIZE]>(data) }
        };

        Self {
            buffer,
            occupied: 0,
        }
    }
}
