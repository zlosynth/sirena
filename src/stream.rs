use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::mpsc;

use crate::samples::{self, Samples};

struct RingBuffer {
    buffer: Samples,
    index: usize,
    req_tx: mpsc::Sender<()>,
    data_rx: mpsc::Receiver<Samples>,
}

impl RingBuffer {
    pub fn new(req_tx: mpsc::Sender<()>, data_rx: mpsc::Receiver<[f32; 32]>) -> Self {
        req_tx.send(()).unwrap();
        Self {
            buffer: samples::zeroed(),
            index: 0,
            req_tx,
            data_rx,
        }
    }

    pub fn pop(&mut self) -> f32 {
        if self.index == 0 {
            self.buffer = self.data_rx.recv().expect("Buffer data sender was closed");
            self.req_tx
                .send(())
                .expect("Buffer request receiver was closed");
        }

        let value = self.buffer[self.index];

        if self.index == 31 {
            self.index = 0;
        } else {
            self.index += 1;
        }

        value
    }
}

pub fn build_output_stream(
    sample_rate: u32,
) -> (cpal::Stream, mpsc::Receiver<()>, mpsc::Sender<[f32; 32]>) {
    let (req_tx, req_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel();

    let mut buffer = RingBuffer::new(req_tx, data_rx);

    #[cfg(any(
        not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
        not(feature = "jack")
    ))]
    let device =
        get_default_output_device().expect("Failed connecting to the default output device");

    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    let device = get_jack_output_device()
        .or_else(get_default_output_device)
        .expect("Failed connecting to both Jack and the default output device");

    let mut supported_configs = device
        .supported_output_configs()
        .expect("Failed querying supported output configuration");

    let config = supported_configs
        .find(|c| c.sample_format() == cpal::SampleFormat::F32 && c.channels() == 2)
        .expect("No suitable output config is available")
        .with_sample_rate(cpal::SampleRate(sample_rate));

    let err_fn = |err| eprintln!("An error occurred on the output stream: {}", err);

    let output_stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(2) {
                    let value = cpal::Sample::from::<f32>(&buffer.pop());
                    frame[0] = value;
                    frame[1] = value;
                }
            },
            err_fn,
        )
        .expect("Failed constructing the output stream");

    (output_stream, req_rx, data_tx)
}

fn get_default_output_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    host.default_output_device()
}

#[cfg(all(
    any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
    feature = "jack"
))]
fn get_jack_output_device() -> Option<cpal::Device> {
    let host_id = cpal::available_hosts()
        .into_iter()
        .find(|id| *id == cpal::HostId::Jack)?;
    let host = cpal::host_from_id(host_id).ok()?;

    host.default_output_device()
}
