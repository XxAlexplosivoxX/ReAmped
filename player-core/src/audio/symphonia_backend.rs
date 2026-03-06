use std::{
    fs::File,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use audioadapter_buffers::number_to_float::InterleavedNumbers;
use rubato::{Fft, Resampler};

use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs::DecoderOptions,
        formats::{FormatOptions, SeekMode, SeekTo},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    },
    default::get_probe,
};

use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use super::{AudioBackend, viz_source::SharedSamples};
use crate::Track;

use ringbuf::RingBuffer;

pub struct SymphoniaBackend {
    samples: SharedSamples,
    playing: Arc<AtomicBool>,
    start: Option<Instant>,
    paused_at: f32,
    volume: Arc<Mutex<f32>>,
    stream: Option<Stream>,
    alive: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    decode_handle: Option<std::thread::JoinHandle<()>>,
}

impl SymphoniaBackend {
    pub fn new(samples: SharedSamples) -> Self {
        Self {
            samples,
            playing: Arc::new(AtomicBool::new(false)),
            start: None,
            paused_at: 0.0,
            volume: Arc::new(Mutex::new(1.0)),
            stream: None,
            alive: Arc::new(AtomicBool::new(true)),
            finished: Arc::new(AtomicBool::new(false)),
            decode_handle: None,
        }
    }

    fn spawn_player(&mut self, path: &Path, seek: f32) {
        self.alive.store(true, Ordering::SeqCst);
        self.finished.store(false, Ordering::SeqCst);
        // const MIN_BUFFER: usize = 4096;
        let samples_viz = self.samples.clone();
        let playing = self.playing.clone();
        let volume = self.volume.clone();

        // let audio_buf: AudioBuffer = Arc::new(Mutex::new(Vec::with_capacity(48_000)));

        playing.store(true, Ordering::SeqCst);
        self.start = Some(Instant::now());

        // ================= CPAL =================
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let mut config: cpal::StreamConfig = device.default_output_config().unwrap().into();

        let output_sr = config.sample_rate as usize;
        let ring = RingBuffer::<f32>::new(output_sr * 4);
        let (mut producer, mut consumer) = ring.split();
        config.channels = 2; // estéreo

        // ================= DECODE THREAD =================
        let finished = self.finished.clone();
        let path = path.to_owned();
        let pl = playing.clone();
        let alive_cl = self.alive.clone();

        let handle = thread::spawn(move || {
            let file = File::open(path).unwrap();
            let mss = MediaSourceStream::new(Box::new(file), Default::default());

            let probed = get_probe()
                .format(
                    &Hint::new(),
                    mss,
                    &FormatOptions::default(),
                    &MetadataOptions::default(),
                )
                .unwrap();

            let mut format = probed.format;
            let track = format.default_track().unwrap();

            let channels = track.codec_params.channels.unwrap().count();
            let input_sr = track.codec_params.sample_rate.unwrap() as usize;

            let mut params = track.codec_params.clone();
            params.sample_rate = Some(output_sr as u32);

            let mut decoder = symphonia::default::get_codecs()
                .make(&params, &DecoderOptions::default())
                .unwrap();

            if seek > 0.0 {
                let _ = format.seek(
                    SeekMode::Accurate,
                    SeekTo::Time {
                        time: seek.into(),
                        track_id: Some(track.id),
                    },
                );
            }

            let chunk_size = 128;
            let mut interleaved = Vec::<f32>::new();

            let mut resampler = Fft::<f32>::new(
                input_sr,
                output_sr,
                chunk_size,
                2,
                channels,
                rubato::FixedSync::Output,
            )
            .unwrap();

            while alive_cl.load(Ordering::SeqCst) {
                if !pl.load(Ordering::SeqCst) {
                    thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                }

                let packet = match format.next_packet() {
                    Ok(p) => p,
                    Err(_) => break,
                };

                let decoded = decoder.decode(&packet).unwrap();
                let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                buf.copy_interleaved_ref(decoded);

                for frame in buf.samples().chunks(channels) {
                    let (l, r) = if channels == 1 {
                        (frame[0], frame[0])
                    } else {
                        (frame[0], frame[1])
                    };

                    interleaved.push(l);
                    interleaved.push(r);

                    let needed = resampler.input_frames_next();

                    if interleaved.len() >= needed * 2 {
                        let input =
                            InterleavedNumbers::new(&interleaved[..needed * 2], 2, needed).unwrap();

                        let output = resampler.process(&input, 0, None).unwrap();
                        let out = output.take_data();

                        while alive_cl.load(Ordering::SeqCst)
                            && producer.len() + out.len() > producer.capacity()
                        {
                            thread::sleep(Duration::from_millis(1));
                        }
                        if !alive_cl.load(Ordering::SeqCst) {
                            break;
                        }
                        let _ = producer.push_slice(&out);

                        interleaved.drain(..needed * 2);
                    }
                    if interleaved.capacity() > 8192 {
                        interleaved.shrink_to(4096);
                    }
                }
            }
            finished.store(true, Ordering::SeqCst);
        });
        self.decode_handle = Some(handle);
        // ================= CPAL STREAM =================
        let stream = device
            .build_output_stream(
                &config,
                move |out: &mut [f32], _| {
                    let vol = *volume.lock().unwrap();
                    for s in out.iter_mut() {
                        if !playing.load(Ordering::SeqCst) {
                            *s = 0.0;
                            continue;
                        }

                        if let Some(sample) = consumer.pop() {
                            *s = sample * vol;
                            let mut viz = samples_viz.lock().unwrap();
                            viz.push(s.clone());
                        } else {
                            *s = 0.0;
                        }
                    }
                },
                |e| eprintln!("audio error: {e}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();
        self.stream = Some(stream);
    }
}

impl AudioBackend for SymphoniaBackend {
    fn load(&mut self, track: &Track) {
        self.stop();
        self.spawn_player(&track.path, 0.0);
    }

    fn play(&mut self) {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        self.playing.store(true, Ordering::SeqCst);
    }

    fn pause(&mut self) {
        if let Some(start) = self.start {
            self.paused_at += start.elapsed().as_secs_f32();
            self.start = None;
        }
        self.samples.lock().unwrap().clear();
        self.playing.store(false, Ordering::SeqCst);
    }

    fn stop(&mut self) {
        self.playing.store(false, Ordering::SeqCst);
        self.alive.store(false, Ordering::SeqCst);
        self.start = None;
        self.paused_at = 0.0;
        if let Some(h) = self.decode_handle.take() {
            let _ = h.join();
        }
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.samples.lock().unwrap().clear();
    }

    fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume;
    }

    fn seek(&mut self, path: &Path, seconds: f32) {
        self.stop();
        self.paused_at = seconds;
        self.spawn_player(path, seconds);
    }

    fn position(&self) -> f32 {
        match self.start {
            Some(t) => self.paused_at + t.elapsed().as_secs_f32(),
            None => self.paused_at,
        }
    }

    fn samples(&self) -> SharedSamples {
        self.samples.clone()
    }

    fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }
}
