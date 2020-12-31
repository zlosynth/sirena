#[macro_use]
extern crate graphity;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use graphity::{NodeIndex, NodeWrapper};
use std::sync::mpsc;
use std::thread;

mod oscillator {
    use core::f32::consts::PI;

    use graphity::Node;
    use sirena::{Buffer, SAMPLE_RATE};

    #[derive(Default)]
    pub struct Oscillator {
        phase: f32,
        frequency: Buffer,
        result: Buffer,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Input;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Output;

    impl Node<Buffer> for Oscillator {
        type Consumer = Input;
        type Producer = Output;

        fn write(&mut self, _input: Input, data: Buffer) {
            self.frequency = data;
        }

        fn read(&self, _output: Output) -> Buffer {
            self.result
        }

        fn tick(&mut self) {
            for (i, result) in self.result.iter_mut().enumerate() {
                *result = (self.phase * self.frequency[i] * 2.0 * PI / SAMPLE_RATE).sin();
                self.phase = (self.phase + 1.0) % SAMPLE_RATE;
            }
        }
    }
}

mod generator {
    use graphity::Node;
    use sirena::{Buffer, BUFFER_SIZE};

    pub struct Generator(Buffer);

    impl Generator {
        pub fn new(value: f32) -> Self {
            Self([value; BUFFER_SIZE])
        }
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Input;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Output;

    impl Node<Buffer> for Generator {
        type Consumer = Input;
        type Producer = Output;

        fn write(&mut self, _input: Input, data: Buffer) {
            self.0 = data;
        }

        fn read(&self, _output: Output) -> Buffer {
            self.0
        }
    }
}

mod sink {
    use graphity::Node;
    use sirena::Buffer;

    #[derive(Default)]
    pub struct Sink(Buffer);

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Input;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Output;

    impl Node<Buffer> for Sink {
        type Consumer = Input;
        type Producer = Output;

        fn write(&mut self, _input: Input, data: Buffer) {
            self.0 = data;
        }

        fn read(&self, _output: Output) -> Buffer {
            self.0
        }
    }
}

mod instrument {
    use super::{generator::Generator, oscillator::Oscillator, sink::Sink};
    use sirena::Buffer;
    graphity!(Instrument<Buffer>; Generator, Oscillator, Sink);
}
use instrument::Instrument;

fn main() {
    struct Buffer {
        buffer: Option<[f32; 32]>,
        index: usize,
        req_tx: mpsc::Sender<()>,
        data_rx: mpsc::Receiver<[f32; 32]>,
    }

    impl Buffer {
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
            std::thread::sleep(std::time::Duration::from_millis(1000));
            (buffer, req_rx, data_tx)
        }

        pub fn pop(&mut self) -> f32 {
            if self.index == 0 {
                let new_data = self.data_rx.recv().unwrap();
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

    let (mut buffer, req_rx, data_tx) = { Buffer::new() };

    thread::spawn(move || {
        let mut instrument = Instrument::new();

        let frequency_generator = instrument.add_node(generator::Generator::new(440.0));
        let main_oscillator = instrument.add_node(oscillator::Oscillator::default());
        let output_sink = instrument.add_node(sink::Sink::default());

        instrument.must_add_edge(
            frequency_generator.producer(generator::Output),
            main_oscillator.consumer(oscillator::Input),
        );
        instrument.must_add_edge(
            main_oscillator.producer(oscillator::Output),
            output_sink.consumer(sink::Input),
        );

        for _ in req_rx {
            instrument.tick();
            let data = instrument.node(&output_sink).unwrap().read(sink::Output);
            data_tx.send(data).unwrap();
        }
    });

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

    let stream = device
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
        .unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(20000));
}
