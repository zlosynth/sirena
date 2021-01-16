// TODO:
// - sampled

use gazpatcho::config as c;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::samples::Samples;

pub struct Class;

impl<N, C, P> crate::registration::Module<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(
        &self,
        id: String,
        _data: HashMap<String, gazpatcho::model::Value>,
    ) -> (Box<dyn crate::Widget>, N) {
        let buffer = {
            let mut data: [std::mem::MaybeUninit<f32>; 2000] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            for elem in &mut data[..] {
                unsafe {
                    std::ptr::write(elem.as_mut_ptr(), 0.0);
                }
            }
            unsafe { std::mem::transmute::<_, [f32; 2000]>(data) }
        };
        let buffer = Arc::new(Mutex::new(buffer));
        let buffer_len = Arc::new(Mutex::new(2000));
        (
            Box::new(Module {
                id,
                buffer: Arc::clone(&buffer),
                buffer_len: Arc::clone(&buffer_len),
                join_handle: None,
                stop_tx: None,
            }),
            Node {
                ..Node::new(buffer, buffer_len)
            }
            .into(),
        )
    }

    fn template(&self) -> c::NodeTemplate {
        c::NodeTemplate {
            label: "Scope".to_owned(),
            class: "scope".to_owned(),
            display_heading: false,
            pins: vec![c::Pin {
                label: "In".to_owned(),
                class: "in".to_owned(),
                direction: c::Input,
            }],
            widgets: vec![c::Canvas {
                key: "scope".to_owned(),
                size: [400.0, 200.0],
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
    buffer: Arc<Mutex<[f32; 2000]>>,
    buffer_len: Arc<Mutex<i32>>,
    stop_tx: Option<mpsc::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl crate::registration::Widget for Module {
    fn register_ui_tx(&mut self, ui_tx: mpsc::Sender<gazpatcho::request::Request>) {
        let (stop_tx, stop_rx) = mpsc::channel();

        let id = self.id.clone();

        let mut prev_y = 0;

        let buffer = Arc::clone(&self.buffer);
        let buffer_len = Arc::clone(&self.buffer_len);

        let join_handle = thread::spawn(move || loop {
            let mut wave = Vec::new();
            let buffer = { buffer.lock().unwrap().clone() };
            let buffer_len = { *buffer_len.lock().unwrap() } as usize;
            for i in 0..400 {
                let new_y = -buffer[i * buffer_len / 400] as i32 + 100;

                if i != 0 {
                    if prev_y < new_y {
                        for y in prev_y + 1..=new_y {
                            wave.push((i as f32, y as f32));
                        }
                    } else if prev_y > new_y {
                        for y in new_y..prev_y {
                            wave.push((i as f32, y as f32));
                        }
                    } else {
                        wave.push((i as f32, new_y as f32));
                    }
                } else {
                    wave.push((i as f32, new_y as f32));
                }
                prev_y = new_y;
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

impl Drop for Module {
    fn drop(&mut self) {
        if let Some(stop_tx) = &self.stop_tx {
            stop_tx.send(()).unwrap();
            self.join_handle.take().unwrap().join().unwrap();
        }
    }
}

pub struct Node {
    input: Samples,
    // TODO: Introduce ring buffer struct, including the index
    buffer: Arc<Mutex<[f32; 2000]>>,
    buffer_len: Arc<Mutex<i32>>,
    index: usize,

    sum: f32,
    sum_n: i32,

    ticks: i32,
    interval: i32,
    since_tick: i32,
    prev_val: f32,
    awaiting_reset: bool,
}

impl Node {
    fn new(buffer: Arc<Mutex<[f32; 2000]>>, buffer_len: Arc<Mutex<i32>>) -> Self {
        Self {
            input: [0.0; 32],
            buffer,
            buffer_len,
            index: 0,
            sum: 0.0,
            sum_n: 0,
            ticks: 0,
            interval: 48000,
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
        // TODO:
        // 2000
        // should fit 2 intervals
        // 48000 / s
        //
        // 480 / s => has 100 frames
        //
        // need to fit 200 frames to 2000
        //
        // WORKS!

        for i in self.input.iter() {
            self.since_tick += 1;
            if self.prev_val < 0.0 && *i > 0.0 {
                self.ticks += 1;
                self.awaiting_reset = false;
            }

            if self.ticks == 1 {
                self.interval = self.since_tick / 1;
                self.ticks = 0;
                self.since_tick = 0;
            }

            self.prev_val = *i;

            if self.awaiting_reset {
                continue;
            }

            self.sum += *i;
            self.sum_n += 1;

            // TODO: Move out, so it does not change every tick
            let zoom_in = i32::max(1, self.interval * 3 / 2000);
            {
                *self.buffer_len.lock().unwrap() = i32::min(self.interval * 3, 2000);
            }

            if self.sum_n >= zoom_in {
                {
                    let mut buffer = self.buffer.lock().unwrap();
                    buffer[self.index] = self.sum / self.sum_n as f32;
                }
                self.index += 1;
                if self.index == 2000 {
                    self.awaiting_reset = true;
                    self.index = 0;
                }

                self.sum = 0.0;
                self.sum_n = 0;
            }
        }

        // println!("Interval: {:?}", self.interval);
    }
}
