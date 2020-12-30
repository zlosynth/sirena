use sirena::BUFFER_SIZE;

sirena!(Instrument: Generator + Oscillator + Sink + Clock);

const SAMPLE_RATE: f32 = 44800.0;
static TICK_INTERVAL: f32 = BUFFER_SIZE / SAMPLE_RATE;

fn main() {
    let mut instrument = Instrument::new();

    let clock = instrument.add_module(Clock::new(TICK_INTERVAL));
    let generator = instrument.add_module(Generator::new(444.0));
    let oscillator = instrument.add_module(Oscillator::new());
    let sink = instrument.add_module(Sink::new());

    instrument.add_patch(
        clock.output(ClockOut),
        oscillator.input(OscillatorClockIn),
    );
    instrument.add_patch(
        generator.output(GeneratorOut),
        oscillator.input(OscillatorFrequencyIn),
    );
    instrument.add_patch(
        oscillator.output(OscillatorOut),
        sink.input(SinkIn),
    );

    let callback = |buffer, frames| {
        instrument.tick();
        *buffer = instrument.module(&sink).unwrap().read(SinkOut);
    };
}
