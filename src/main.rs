#![allow(dead_code)]
use std::env;
use std::f32::consts::PI;
use std::fs;
use std::io;
use std::path::PathBuf;

const DEFAULT_SAMPLE_RATE: u32 = 44_100;
const DEFAULT_VOLUME: f32 = 0.82;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PresetOption {
    Single(Preset),
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Preset {
    Soft,
    Classic,
    Crisp,
    Blue,
    Balanced,
    Retro,
}

#[derive(Clone, Copy, Debug)]
struct SoundSpec {
    file_name: &'static str,
    kind: SoundKind,
    duration_ms: u32,
    base_hz: f32,
    noise: f32,
    body: f32,
    brightness: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SoundKind {
    BlueKey,
    EnterKiang,
    BackspaceWhoosh,
    DeleteSnap,
    SpaceBar,
    RetroBackspace,
    TypewriterEnter,
}

#[derive(Debug)]
struct Options {
    out_dir: PathBuf,
    preset: PresetOption,
    sample_rate: u32,
    volume: f32,
    trace_from: Option<PathBuf>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let options = parse_args(env::args().skip(1))?;
    fs::create_dir_all(&options.out_dir)?;

    match options.preset {
        PresetOption::Single(preset) => {
            generate_preset(preset, &options.out_dir, &options)?;
        }
        PresetOption::All => {
            let presets = [
                (Preset::Classic, "classic"),
                (Preset::Soft, "soft"),
                (Preset::Crisp, "crisp"),
                (Preset::Blue, "blue"),
                (Preset::Retro, "retro"),
                (Preset::Balanced, "balanced"),
            ];
            for &(preset, dir_name) in &presets {
                let preset_dir = options.out_dir.join(dir_name);
                fs::create_dir_all(&preset_dir)?;
                generate_preset(preset, &preset_dir, &options)?;
            }
        }
    }

    Ok(())
}

fn generate_preset(
    preset: Preset,
    target_dir: &std::path::Path,
    options: &Options,
) -> io::Result<()> {
    for (index, spec) in specs_for_preset(preset).iter().enumerate() {
        let wav = build_output_wav(spec, options, preset, index as u32 + 1)?;
        let path = target_dir.join(spec.file_name);
        fs::write(&path, wav)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn build_output_wav(
    spec: &SoundSpec,
    options: &Options,
    preset: Preset,
    seed: u32,
) -> io::Result<Vec<u8>> {
    if let Some(reference_dir) = &options.trace_from {
        let ref_file_name = if spec.file_name == "0.wav" {
            "1.wav"
        } else {
            spec.file_name
        };
        let reference_path = reference_dir.join(ref_file_name);
        match fs::read(&reference_path) {
            Ok(reference_wav) => {
                return build_traced_wav_bytes(
                    &reference_wav,
                    preset,
                    spec.file_name,
                    options.volume,
                    seed,
                );
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }

    Ok(build_wav_bytes(
        spec,
        options.sample_rate,
        options.volume,
        seed,
    ))
}

fn should_trace_reference(_spec: &SoundSpec) -> bool {
    true
}

fn parse_args<I>(args: I) -> io::Result<Options>
where
    I: IntoIterator<Item = String>,
{
    let mut out_dir = PathBuf::from("wavs");
    let mut preset = PresetOption::Single(Preset::Classic);
    let mut sample_rate = DEFAULT_SAMPLE_RATE;
    let mut volume = DEFAULT_VOLUME;
    let mut trace_from = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-o" | "--out" => {
                let value = next_arg(&mut iter, &arg)?;
                out_dir = PathBuf::from(value);
            }
            "-p" | "--preset" => {
                let value = next_arg(&mut iter, &arg)?;
                preset = if value.to_ascii_lowercase() == "all" {
                    PresetOption::All
                } else {
                    PresetOption::Single(Preset::parse(&value)?)
                };
            }
            "--sample-rate" => {
                let value = next_arg(&mut iter, &arg)?;
                sample_rate = parse_sample_rate(&value)?;
            }
            "-v" | "--volume" => {
                let value = next_arg(&mut iter, &arg)?;
                volume = parse_volume(&value)?;
            }
            "--trace-from" | "--trace" => {
                let value = next_arg(&mut iter, &arg)?;
                trace_from = Some(PathBuf::from(value));
            }
            _ => {
                return Err(invalid(format!("unknown argument: {arg}")));
            }
        }
    }

    Ok(Options {
        out_dir,
        preset,
        sample_rate,
        volume,
        trace_from,
    })
}

impl Preset {
    fn parse(value: &str) -> io::Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "soft" => Ok(Self::Soft),
            "classic" => Ok(Self::Classic),
            "crisp" => Ok(Self::Crisp),
            "blue" | "mx-blue" | "cherry-blue" | "青軸" => Ok(Self::Blue),
            "balanced" => Ok(Self::Balanced),
            "retro" | "typewriter" => Ok(Self::Retro),
            _ => Err(invalid(format!(
                "unknown preset: {value}; expected soft, classic, crisp, blue, balanced, or retro"
            ))),
        }
    }
}

fn specs_for_preset(preset: Preset) -> Vec<SoundSpec> {
    let mut specs = vec![
        SoundSpec {
            file_name: "0.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 43,
            base_hz: 1_280.0,
            noise: 0.86,
            body: 0.44,
            brightness: 0.92,
        },
        SoundSpec {
            file_name: "1.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 34,
            base_hz: 1_450.0,
            noise: 0.92,
            body: 0.38,
            brightness: 1.05,
        },
        SoundSpec {
            file_name: "2.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 35,
            base_hz: 1_520.0,
            noise: 0.93,
            body: 0.36,
            brightness: 1.06,
        },
        SoundSpec {
            file_name: "3.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 36,
            base_hz: 1_590.0,
            noise: 0.94,
            body: 0.35,
            brightness: 1.07,
        },
        SoundSpec {
            file_name: "4.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 37,
            base_hz: 1_660.0,
            noise: 0.95,
            body: 0.34,
            brightness: 1.08,
        },
        SoundSpec {
            file_name: "5.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 38,
            base_hz: 1_730.0,
            noise: 0.96,
            body: 0.33,
            brightness: 1.09,
        },
        SoundSpec {
            file_name: "6.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 39,
            base_hz: 1_800.0,
            noise: 0.97,
            body: 0.32,
            brightness: 1.10,
        },
        SoundSpec {
            file_name: "7.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 40,
            base_hz: 1_870.0,
            noise: 0.98,
            body: 0.31,
            brightness: 1.11,
        },
        SoundSpec {
            file_name: "8.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 41,
            base_hz: 1_940.0,
            noise: 0.99,
            body: 0.30,
            brightness: 1.12,
        },
        SoundSpec {
            file_name: "9.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 42,
            base_hz: 2_010.0,
            noise: 1.00,
            body: 0.29,
            brightness: 1.13,
        },
        SoundSpec {
            file_name: "enter.wav",
            kind: SoundKind::EnterKiang,
            duration_ms: 118,
            base_hz: 310.0,
            noise: 0.72,
            body: 0.88,
            brightness: 0.72,
        },
        SoundSpec {
            file_name: "delete.wav",
            kind: SoundKind::DeleteSnap,
            duration_ms: 54,
            base_hz: 1_250.0,
            noise: 0.90,
            body: 0.24,
            brightness: 1.22,
        },
        SoundSpec {
            file_name: "backspace.wav",
            kind: SoundKind::BackspaceWhoosh,
            duration_ms: 92,
            base_hz: 1_760.0,
            noise: 0.86,
            body: 0.34,
            brightness: 0.92,
        },
        SoundSpec {
            file_name: "space.wav",
            kind: SoundKind::SpaceBar,
            duration_ms: 72,
            base_hz: 410.0,
            noise: 0.62,
            body: 0.70,
            brightness: 0.42,
        },
    ];

    if preset == Preset::Balanced {
        apply_balanced_shape(&mut specs);
        return specs;
    }
    if preset == Preset::Blue {
        apply_blue_switch_shape(&mut specs);
        return specs;
    }
    if preset == Preset::Retro {
        apply_retro_typewriter_shape(&mut specs);
        return specs;
    }

    let (duration_scale, pitch_scale, noise_scale, body_scale, brightness_scale) = match preset {
        Preset::Soft => (1.15, 0.92, 0.55, 0.92, 0.55),
        Preset::Classic => (1.0, 1.0, 1.0, 1.0, 1.0),
        Preset::Crisp => (0.82, 1.12, 1.18, 0.78, 1.25),
        Preset::Blue => unreachable!(),
        Preset::Balanced => unreachable!(),
        Preset::Retro => unreachable!(),
    };

    for spec in &mut specs {
        spec.duration_ms = ((spec.duration_ms as f32) * duration_scale).round() as u32;
        spec.base_hz *= pitch_scale;
        spec.noise *= noise_scale;
        spec.body *= body_scale;
        spec.brightness *= brightness_scale;
    }

    specs
}

fn apply_retro_typewriter_shape(specs: &mut [SoundSpec]) {
    for (index, spec) in specs.iter_mut().enumerate() {
        match spec.file_name {
            "0.wav" | "1.wav" | "2.wav" | "3.wav" | "4.wav" | "5.wav" | "6.wav" | "7.wav"
            | "8.wav" | "9.wav" => {
                spec.kind = SoundKind::BlueKey;
                spec.duration_ms = 85 + ((index as u32 * 4) % 20);
                spec.base_hz = 850.0 + index as f32 * 50.0;
                spec.noise = 0.85;
                spec.body = 0.90;
                spec.brightness = 1.10;
            }
            "enter.wav" => {
                spec.kind = SoundKind::TypewriterEnter;
                spec.duration_ms = 350;
                spec.base_hz = 380.0;
                spec.noise = 0.75;
                spec.body = 1.20;
                spec.brightness = 1.30;
            }
            "delete.wav" => {
                spec.kind = SoundKind::DeleteSnap;
                spec.duration_ms = 90;
                spec.base_hz = 1_200.0;
                spec.noise = 0.80;
                spec.body = 0.50;
                spec.brightness = 1.10;
            }
            "backspace.wav" => {
                spec.kind = SoundKind::RetroBackspace;
                spec.duration_ms = 180;
                spec.base_hz = 1_500.0;
                spec.noise = 0.90;
                spec.body = 0.45;
                spec.brightness = 1.40;
            }
            "space.wav" => {
                spec.kind = SoundKind::SpaceBar;
                spec.duration_ms = 130;
                spec.base_hz = 220.0;
                spec.noise = 0.70;
                spec.body = 1.30;
                spec.brightness = 0.70;
            }
            _ => {}
        }
    }
}

fn apply_blue_switch_shape(specs: &mut [SoundSpec]) {
    for (index, spec) in specs.iter_mut().enumerate() {
        match spec.file_name {
            "0.wav" | "1.wav" | "2.wav" | "3.wav" | "4.wav" | "5.wav" | "6.wav" | "7.wav"
            | "8.wav" | "9.wav" => {
                spec.duration_ms = 78 + ((index as u32 * 3) % 15);
                spec.base_hz = 2_240.0 + index as f32 * 95.0;
                spec.noise = 0.52 + index as f32 * 0.012;
                spec.body = 0.82 - index as f32 * 0.018;
                spec.brightness = 1.28 + index as f32 * 0.018;
            }
            "enter.wav" => {
                spec.duration_ms = 145;
                spec.base_hz = 265.0;
                spec.noise = 0.70;
                spec.body = 1.12;
                spec.brightness = 0.90;
            }
            "delete.wav" => {
                spec.duration_ms = 86;
                spec.base_hz = 2_260.0;
                spec.noise = 0.58;
                spec.body = 0.64;
                spec.brightness = 1.25;
            }
            "backspace.wav" => {
                spec.duration_ms = 108;
                spec.base_hz = 1_620.0;
                spec.noise = 0.50;
                spec.body = 0.70;
                spec.brightness = 1.05;
            }
            "space.wav" => {
                spec.duration_ms = 112;
                spec.base_hz = 235.0;
                spec.noise = 0.58;
                spec.body = 1.08;
                spec.brightness = 0.64;
            }
            _ => {}
        }
    }
}

fn apply_balanced_shape(specs: &mut [SoundSpec]) {
    for spec in specs {
        match spec.file_name {
            "0.wav" => tune_reference_key(spec, 122, 1_080.0, 0.70, 0.50, 0.78),
            "1.wav" => tune_reference_key(spec, 197, 1_260.0, 0.84, 0.44, 0.88),
            "2.wav" => tune_reference_key(spec, 139, 1_340.0, 0.90, 0.42, 0.92),
            "3.wav" => tune_reference_key(spec, 156, 1_120.0, 0.72, 0.40, 0.74),
            "4.wav" => tune_reference_key(spec, 111, 1_430.0, 0.92, 0.38, 0.96),
            "5.wav" => tune_reference_key(spec, 199, 1_520.0, 0.82, 0.42, 0.90),
            "6.wav" => tune_reference_key(spec, 134, 1_760.0, 1.02, 0.36, 1.08),
            "7.wav" => tune_reference_key(spec, 137, 1_480.0, 0.74, 0.35, 0.86),
            "8.wav" => tune_reference_key(spec, 172, 1_210.0, 0.78, 0.40, 0.82),
            "9.wav" => tune_reference_key(spec, 97, 1_690.0, 1.00, 0.34, 1.08),
            "enter.wav" => {
                spec.duration_ms = 485;
                spec.base_hz = 285.0;
                spec.noise = 0.86;
                spec.body = 1.10;
                spec.brightness = 0.88;
            }
            "delete.wav" => {
                spec.duration_ms = 175;
                spec.base_hz = 980.0;
                spec.noise = 0.86;
                spec.body = 0.34;
                spec.brightness = 0.94;
            }
            "backspace.wav" => {
                spec.duration_ms = 175;
                spec.base_hz = 640.0;
                spec.noise = 0.42;
                spec.body = 0.54;
                spec.brightness = 0.92;
            }
            "space.wav" => {
                spec.duration_ms = 168;
                spec.base_hz = 390.0;
                spec.noise = 0.72;
                spec.body = 0.82;
                spec.brightness = 0.58;
            }
            _ => {}
        }
    }
}

fn tune_reference_key(
    spec: &mut SoundSpec,
    duration_ms: u32,
    base_hz: f32,
    noise: f32,
    body: f32,
    brightness: f32,
) {
    spec.duration_ms = duration_ms;
    spec.base_hz = base_hz;
    spec.noise = noise;
    spec.body = body;
    spec.brightness = brightness;
}

fn build_wav_bytes(spec: &SoundSpec, sample_rate: u32, volume: f32, seed: u32) -> Vec<u8> {
    let samples = synthesize_samples(spec, sample_rate, volume, seed);
    write_pcm16_mono_wav(&samples, sample_rate)
}

fn build_traced_wav_bytes(
    reference_wav: &[u8],
    preset: Preset,
    file_name: &str,
    volume: f32,
    seed: u32,
) -> io::Result<Vec<u8>> {
    let reference = parse_pcm16_wav_mono(reference_wav)?;
    let processed_samples = process_reference_samples(
        &reference.samples,
        reference.sample_rate,
        preset,
        file_name,
        seed,
    );
    let speed = match preset {
        Preset::Balanced => 1.015,
        Preset::Soft => 0.90,
        Preset::Crisp => 1.15,
        Preset::Blue => 1.05,
        Preset::Classic => 1.00,
        Preset::Retro => {
            if file_name == "enter.wav" || file_name == "backspace.wav" || file_name == "delete.wav"
            {
                1.0
            } else {
                0.78
            }
        }
    };

    let resampled_samples = resample(&processed_samples, speed);
    let normalized = normalize_to_i16(&resampled_samples, volume);
    Ok(write_pcm16_mono_wav(&normalized, reference.sample_rate))
}

fn resample(samples: &[f32], speed: f32) -> Vec<f32> {
    if (speed - 1.0).abs() < f32::EPSILON || speed <= 0.0 || samples.is_empty() {
        return samples.to_vec();
    }
    let new_len = (samples.len() as f32 / speed) as usize;
    let mut processed = vec![0.0; new_len];
    for i in 0..new_len {
        let src_idx = i as f32 * speed;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(samples.len() - 1);
        let frac = src_idx - idx0 as f32;
        if idx0 < samples.len() {
            processed[i] = samples[idx0] * (1.0 - frac) + samples[idx1] * frac;
        }
    }
    processed
}

fn process_reference_samples(
    samples: &[f32],
    sample_rate: u32,
    preset: Preset,
    file_name: &str,
    seed: u32,
) -> Vec<f32> {
    match preset {
        Preset::Balanced => {
            let mut processed = vec![0.0; samples.len()];
            if !samples.is_empty() {
                processed[0] = samples[0];
                for i in 1..samples.len() {
                    let hp = samples[i] - samples[i - 1];
                    processed[i] = samples[i] * 0.97 + hp * 0.03;
                }
            }
            processed
        }
        Preset::Soft => {
            let mut processed = vec![0.0; samples.len()];
            for i in 0..samples.len() {
                let mut sum = 0.0;
                let mut count = 0;
                for j in -3..=3 {
                    let idx = i as i32 + j;
                    if idx >= 0 && idx < samples.len() as i32 {
                        sum += samples[idx as usize];
                        count += 1;
                    }
                }
                processed[i] = sum / count as f32 * 0.75;
            }
            processed
        }
        Preset::Crisp => {
            let mut processed = vec![0.0; samples.len()];
            if !samples.is_empty() {
                processed[0] = samples[0];
                for i in 1..samples.len() {
                    let hp = samples[i] - samples[i - 1];
                    processed[i] = (samples[i] * 0.4 + hp * 1.2) * 1.25;
                }
            }
            processed
        }
        Preset::Blue => {
            let mut processed = samples.to_vec();
            if !samples.is_empty() {
                processed[0] = samples[0];
                for i in 1..samples.len() {
                    processed[i] = samples[i] * 1.1 - samples[i - 1] * 0.2;
                }
            }

            let mut attack_idx = 0;
            for (i, &s) in samples.iter().enumerate() {
                if s.abs() > 0.03 {
                    attack_idx = i;
                    break;
                }
            }
            let mut rng = Lcg::new(seed ^ 0x3d7f);

            let click_len = ((sample_rate as f32) * 0.006).round() as usize;
            for offset in 0..click_len {
                let idx = attack_idx + offset;
                if idx < processed.len() {
                    let t = offset as f32 / sample_rate as f32;
                    let noise = rng.next_signed();
                    let hp_noise = noise * (-t * 800.0).exp();
                    let click_pop = sin_hz(3200.0, t) * (-t * 400.0).exp() * 0.6;
                    let spring = sin_hz(4500.0, t) * (-t * 220.0).exp() * 0.25;
                    processed[idx] += (hp_noise * 0.7 + click_pop + spring) * 0.8;
                }
            }

            let release_offset = ((sample_rate as f32) * 0.022).round() as usize;
            let release_idx = attack_idx + release_offset;
            for offset in 0..click_len {
                let idx = release_idx + offset;
                if idx < processed.len() {
                    let t = offset as f32 / sample_rate as f32;
                    let noise = rng.next_signed();
                    let hp_noise = noise * (-t * 1000.0).exp() * 0.3;
                    let click_pop = sin_hz(3400.0, t) * (-t * 500.0).exp() * 0.3;
                    processed[idx] += (hp_noise + click_pop) * 0.4;
                }
            }
            processed
        }
        Preset::Classic => {
            let mut processed = vec![0.0; samples.len()];
            if !samples.is_empty() {
                processed[0] = samples[0];
                for i in 1..samples.len() {
                    let hp = samples[i] - samples[i - 1];
                    processed[i] = samples[i] * 0.85 + hp * 0.25;
                }
            }
            processed
        }
        Preset::Retro => {
            if file_name == "enter.wav" {
                let spec = SoundSpec {
                    file_name: "enter.wav",
                    kind: SoundKind::TypewriterEnter,
                    duration_ms: 350,
                    base_hz: 380.0,
                    noise: 0.75,
                    body: 1.20,
                    brightness: 1.30,
                };
                let sample_count = ((sample_rate as f32) * (spec.duration_ms as f32 / 1000.0))
                    .round()
                    .max(1.0) as usize;
                let mut rng = Lcg::new(seed ^ hash_name(spec.file_name));
                let mut raw_samples = Vec::with_capacity(sample_count);
                let mut previous_noise = 0.0;
                for index in 0..sample_count {
                    let t = index as f32 / sample_rate as f32;
                    let progress = if sample_count <= 1 {
                        1.0
                    } else {
                        index as f32 / (sample_count - 1) as f32
                    };
                    let noise = rng.next_signed();
                    let high_pass_noise = noise - previous_noise * 0.58;
                    previous_noise = noise;
                    raw_samples.push(sample_typewriter_enter(
                        &spec,
                        t,
                        progress,
                        noise,
                        high_pass_noise,
                    ));
                }
                return raw_samples;
            }
            if file_name == "backspace.wav" || file_name == "delete.wav" {
                let spec = SoundSpec {
                    file_name: "backspace.wav",
                    kind: SoundKind::RetroBackspace,
                    duration_ms: 180,
                    base_hz: 1500.0,
                    noise: 0.90,
                    body: 0.45,
                    brightness: 1.40,
                };
                let sample_count = ((sample_rate as f32) * (spec.duration_ms as f32 / 1000.0))
                    .round()
                    .max(1.0) as usize;
                let mut rng = Lcg::new(seed ^ hash_name(spec.file_name));
                let mut raw_samples = Vec::with_capacity(sample_count);
                let mut previous_noise = 0.0;
                for index in 0..sample_count {
                    let t = index as f32 / sample_rate as f32;
                    let progress = if sample_count <= 1 {
                        1.0
                    } else {
                        index as f32 / (sample_count - 1) as f32
                    };
                    let noise = rng.next_signed();
                    let high_pass_noise = noise - previous_noise * 0.58;
                    previous_noise = noise;
                    raw_samples.push(sample_retro_backspace(
                        &spec,
                        t,
                        progress,
                        noise,
                        high_pass_noise,
                    ));
                }
                return raw_samples;
            }

            samples.to_vec()
        }
    }
}

fn synthesize_traced_samples(
    reference_samples: &[f32],
    sample_rate: u32,
    volume: f32,
    seed: u32,
) -> Vec<i16> {
    let mut rng = Lcg::new(seed ^ 0x75c1_11u32);
    let envelope = trace_envelope(reference_samples);
    let max_envelope = envelope
        .iter()
        .fold(0.0_f32, |max, sample| max.max(*sample));
    if max_envelope <= f32::EPSILON {
        return vec![0; reference_samples.len()];
    }

    let min_spacing = ((sample_rate as f32) * 0.006).round().max(1.0) as usize;
    let mut last_event: Option<usize> = None;
    let mut raw_samples = vec![0.0_f32; reference_samples.len()];

    for index in 1..envelope.len().saturating_sub(1) {
        let current = envelope[index];
        if current < max_envelope * 0.08 {
            continue;
        }

        let previous = envelope[index - 1];
        let next = envelope[index + 1];
        let is_rising_edge = current > previous * 1.28 && current > max_envelope * 0.12;
        let is_local_peak = current >= previous && current > next && current > max_envelope * 0.18;
        if !(is_rising_edge || is_local_peak)
            || last_event.is_some_and(|last| index.saturating_sub(last) < min_spacing)
        {
            continue;
        }

        let strength = (current / max_envelope).clamp(0.12, 1.0);
        let color = 0.82 + rng.next_signed() * 0.18;
        add_trace_click_response(
            &mut raw_samples,
            &envelope,
            max_envelope,
            index,
            sample_rate,
            strength,
            color,
            &mut rng,
        );
        last_event = Some(index);
    }

    if raw_samples
        .iter()
        .all(|sample| sample.abs() <= f32::EPSILON)
    {
        if let Some((index, value)) = envelope
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            add_trace_click_response(
                &mut raw_samples,
                &envelope,
                max_envelope,
                index,
                sample_rate,
                (*value / max_envelope).clamp(0.12, 1.0),
                1.0,
                &mut rng,
            );
        }
    }

    normalize_to_i16(&raw_samples, volume)
}

fn trace_envelope(reference_samples: &[f32]) -> Vec<f32> {
    let mut envelope = 0.0;
    let mut values = Vec::with_capacity(reference_samples.len());
    for sample in reference_samples {
        let rectified = sample.abs();
        if rectified > envelope {
            envelope = envelope * 0.22 + rectified * 0.78;
        } else {
            envelope = envelope * 0.84 + rectified * 0.16;
        }
        values.push(envelope);
    }
    values
}

fn add_trace_click_response(
    raw_samples: &mut [f32],
    envelope: &[f32],
    max_envelope: f32,
    start: usize,
    sample_rate: u32,
    strength: f32,
    color: f32,
    rng: &mut Lcg,
) {
    let response_len = ((sample_rate as f32) * 0.014).round().max(1.0) as usize;
    let mut previous_noise = 0.0;
    let base = 520.0 + color * 120.0;
    let ping = 2_650.0 + color * 650.0;

    for offset in 0..response_len {
        let index = start + offset;
        if index >= raw_samples.len() {
            break;
        }

        let t = offset as f32 / sample_rate as f32;
        let progress = offset as f32 / response_len as f32;
        let noise = rng.next_signed();
        let high = noise - previous_noise * 0.82;
        previous_noise = noise;
        let envelope_gate = if max_envelope > f32::EPSILON {
            (envelope[index] / max_envelope).clamp(0.0, 1.0).powf(0.65)
        } else {
            0.0
        };

        let switch_leaf = high * (-t * 720.0).exp() * 0.88;
        let jacket = high.signum() * gaussian(progress, 0.045, 0.030) * 0.18;
        let clack = (sin_hz(base, t) * 0.045 + sin_hz(base * 1.55, t) * 0.020) * (-t * 145.0).exp();
        let spring = sin_hz(ping, t) * 0.022 * (-t * 220.0).exp();
        raw_samples[index] += (switch_leaf + jacket + clack + spring) * strength * envelope_gate;
    }
}

fn synthesize_samples(spec: &SoundSpec, sample_rate: u32, volume: f32, seed: u32) -> Vec<i16> {
    let sample_count = ((sample_rate as f32) * (spec.duration_ms as f32 / 1000.0))
        .round()
        .max(1.0) as usize;
    let mut rng = Lcg::new(seed ^ hash_name(spec.file_name));
    let mut raw_samples = Vec::with_capacity(sample_count);
    let mut previous_noise = 0.0;

    for index in 0..sample_count {
        let t = index as f32 / sample_rate as f32;
        let progress = if sample_count <= 1 {
            1.0
        } else {
            index as f32 / (sample_count - 1) as f32
        };

        let noise = rng.next_signed();
        let high_pass_noise = noise - previous_noise * 0.58;
        previous_noise = noise;

        raw_samples.push(match spec.kind {
            SoundKind::BlueKey => sample_blue_key(spec, t, progress, noise, high_pass_noise),
            SoundKind::EnterKiang => sample_enter_kiang(spec, t, progress, high_pass_noise),
            SoundKind::BackspaceWhoosh => {
                sample_backspace_whoosh(spec, t, progress, noise, high_pass_noise)
            }
            SoundKind::DeleteSnap => sample_delete_snap(spec, t, progress, high_pass_noise),
            SoundKind::SpaceBar => sample_space_bar(spec, t, progress, high_pass_noise),
            SoundKind::RetroBackspace => {
                sample_retro_backspace(spec, t, progress, noise, high_pass_noise)
            }
            SoundKind::TypewriterEnter => {
                sample_typewriter_enter(spec, t, progress, noise, high_pass_noise)
            }
        });
    }

    normalize_to_i16(&raw_samples, volume)
}

fn normalize_to_i16(raw_samples: &[f32], volume: f32) -> Vec<i16> {
    let peak = raw_samples
        .iter()
        .fold(0.0_f32, |max, sample| max.max(sample.abs()));
    let normalized_volume = volume.clamp(0.0, 1.0);
    let gain = if peak > 0.0 {
        normalized_volume * 0.90 / peak
    } else {
        0.0
    };

    raw_samples
        .iter()
        .map(|sample| {
            let value = (sample * gain * i16::MAX as f32).round();
            value.clamp(i16::MIN as f32, i16::MAX as f32) as i16
        })
        .collect()
}

struct Pcm16MonoWav {
    sample_rate: u32,
    samples: Vec<f32>,
}

fn parse_pcm16_wav_mono(wav: &[u8]) -> io::Result<Pcm16MonoWav> {
    if wav.len() < 12 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err(invalid("reference is not a RIFF/WAVE file"));
    }

    let (fmt_offset, fmt_size) =
        find_chunk(wav, b"fmt ").ok_or_else(|| invalid("reference wav missing fmt chunk"))?;
    let (data_offset, data_size) =
        find_chunk(wav, b"data").ok_or_else(|| invalid("reference wav missing data chunk"))?;
    if fmt_size < 16 {
        return Err(invalid("reference wav fmt chunk is too small"));
    }

    let audio_format = read_u16(wav, fmt_offset)?;
    let channels = read_u16(wav, fmt_offset + 2)?;
    let sample_rate = read_u32(wav, fmt_offset + 4)?;
    let bits_per_sample = read_u16(wav, fmt_offset + 14)?;
    if audio_format != 1 || bits_per_sample != 16 || channels == 0 {
        return Err(invalid("only PCM 16-bit reference wav files are supported"));
    }

    let bytes_per_frame = channels as usize * 2;
    if data_size == 0 || data_size % bytes_per_frame != 0 {
        return Err(invalid("reference wav data size is invalid"));
    }

    let frames = data_size / bytes_per_frame;
    let mut samples = Vec::with_capacity(frames);
    for frame in 0..frames {
        let mut sum = 0.0_f32;
        for channel in 0..channels as usize {
            let offset = data_offset + (frame * channels as usize + channel) * 2;
            let sample =
                i16::from_le_bytes([wav[offset], wav[offset + 1]]) as f32 / i16::MAX as f32;
            sum += sample;
        }
        samples.push(sum / channels as f32);
    }

    Ok(Pcm16MonoWav {
        sample_rate,
        samples,
    })
}

fn find_chunk(wav: &[u8], chunk_id: &[u8; 4]) -> Option<(usize, usize)> {
    let mut offset = 12;
    while offset + 8 <= wav.len() {
        let size = i32::from_le_bytes([
            wav[offset + 4],
            wav[offset + 5],
            wav[offset + 6],
            wav[offset + 7],
        ]);
        if size < 0 {
            return None;
        }

        let size = size as usize;
        let data_offset = offset + 8;
        if data_offset + size > wav.len() {
            return None;
        }

        if &wav[offset..offset + 4] == chunk_id {
            return Some((data_offset, size));
        }

        offset = data_offset + size + (size % 2);
    }
    None
}

fn read_u16(data: &[u8], offset: usize) -> io::Result<u16> {
    if offset + 2 > data.len() {
        return Err(invalid("unexpected end of wav"));
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32(data: &[u8], offset: usize) -> io::Result<u32> {
    if offset + 4 > data.len() {
        return Err(invalid("unexpected end of wav"));
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

fn sample_retro_backspace(
    spec: &SoundSpec,
    t: f32,
    progress: f32,
    noise: f32,
    high_pass_noise: f32,
) -> f32 {
    let attack = (progress / 0.005).min(1.0);
    let release = (1.0 - progress).max(0.0).powf(1.8);

    // 1. 復古退格掃擊的前段爆裂瞬態。
    let crack_env = gaussian(progress, 0.010, 0.004);
    let crack_noise = high_pass_noise * spec.noise * spec.brightness * crack_env * 1.5;
    let crack_pop = sin_hz(2400.0, t) * crack_env * 0.8;

    // 2. 下滑頻率掃描，加上短噪音尾巴。
    let sweep_progress = progress.min(1.0);
    let whoosh_freq = spec.base_hz * (1.6 - 1.2 * sweep_progress).max(0.2);
    let whoosh_sine = (sin_hz(whoosh_freq, t) * 0.45 + sin_hz(whoosh_freq * 1.02, t) * 0.2)
        * (-progress * 28.0).exp()
        * spec.brightness;
    let whoosh_noise =
        (high_pass_noise * 0.5 + noise * 0.3) * spec.noise * (-progress * 35.0).exp();

    // 3. 低頻 snap/thud，保留復古機械感。
    let snap_body = sin_hz(280.0, t) * spec.body * (-progress * 65.0).exp();

    (crack_noise + crack_pop + whoosh_sine + whoosh_noise + snap_body) * attack * release
}

fn sample_typewriter_enter(
    spec: &SoundSpec,
    t: f32,
    progress: f32,
    noise: f32,
    high_pass_noise: f32,
) -> f32 {
    let attack = (progress / 0.008).min(1.0);
    let release = (1.0 - progress).max(0.0).powf(1.0);

    // 1. Initial mechanical key clack
    let strike = impact(progress, 0.015, 0.012, 1.2);
    let key_clack = high_pass_noise * spec.noise * strike * 1.3;
    let body_thock = sin_hz(spec.base_hz, t) * strike * spec.body * 0.9;

    // 2. Typewriter bell "ding!" triggered shortly after keypress (around 55ms)
    let bell_start = 0.055;
    let bell_decay = trailing_env(progress, bell_start, 4.5);
    let bell_ring =
        (sin_hz(2850.0, t) * 0.40 + sin_hz(3480.0, t) * 0.25 + sin_hz(4150.0, t) * 0.15)
            * bell_decay
            * spec.brightness;

    // 3. Carriage return mechanical clatter/kachunk
    let clatter_start = 0.12;
    let clatter_env = trailing_env(progress, clatter_start, 15.0) * (1.0 - progress).max(0.0);
    let clatter_noise = (high_pass_noise * 0.22 + noise * 0.15) * clatter_env * spec.noise;
    let clatter_resonance =
        (sin_hz(580.0, t) * 0.12 + sin_hz(880.0, t) * 0.08) * clatter_env * spec.body;

    (key_clack + body_thock + bell_ring + clatter_noise + clatter_resonance) * attack * release
}

fn sample_blue_key(
    spec: &SoundSpec,
    t: f32,
    progress: f32,
    noise: f32,
    high_pass_noise: f32,
) -> f32 {
    let release = (1.0 - progress).max(0.0).powf(1.08);
    let press_click = gaussian(progress, 0.050, 0.0070);
    let click_leaf = gaussian(progress, 0.145, 0.0060);
    let bottom_out = gaussian(progress, 0.305, 0.030);
    let case_tail = (1.0 - (-progress * 22.0).exp()) * (-progress * 2.8).exp() * release;

    let switch_click =
        high_pass_noise * spec.noise * spec.brightness * (press_click * 1.10 + click_leaf * 1.16);
    let jacket_tick =
        high_pass_noise.signum() * spec.brightness * (press_click * 0.26 + click_leaf * 0.42);
    let bottom_clack =
        (high_pass_noise * 0.38 + noise * 0.30) * spec.body * bottom_out * 0.92 * release;
    let case_resonance = (noise * 0.38 + high_pass_noise * 0.12) * spec.body * case_tail * 1.85;
    let keycap_rattle = high_pass_noise * spec.noise * (bottom_out + click_leaf * 0.35) * 0.10;
    let spring_ping = (sin_hz(2_950.0, t) * 0.018 + sin_hz(4_250.0, t) * 0.012)
        * spec.brightness
        * (-progress * 10.0).exp()
        * release;

    switch_click + jacket_tick + bottom_clack + case_resonance + keycap_rattle + spring_ping
}

fn sample_backspace_whoosh(
    spec: &SoundSpec,
    t: f32,
    progress: f32,
    noise: f32,
    high_pass_noise: f32,
) -> f32 {
    let attack = (progress / 0.030).min(1.0);
    let release = (1.0 - progress).max(0.0).powf(1.55);
    let sweep_env = gaussian(progress, 0.38, 0.24) * release;
    let frequency = spec.base_hz * (1.82 - 1.36 * progress).max(0.30);
    let whistle = sin_hz(frequency, t) * 0.52 + sin_hz(frequency * 1.013, t) * 0.22;
    let airy = (noise * 0.68 + high_pass_noise * 0.18) * spec.noise;
    let tail = sin_hz(230.0, t) * spec.body * (-progress * 5.8).exp();

    (whistle * spec.brightness + airy * sweep_env + tail) * attack * release
}

fn sample_enter_kiang(spec: &SoundSpec, t: f32, progress: f32, high_pass_noise: f32) -> f32 {
    let release = (1.0 - progress).max(0.0).powf(0.86);
    let strike = impact(progress, 0.022, 0.020, 1.08);
    let platen = impact(progress, 0.105, 0.038, 0.72);
    let carriage = impact(progress, 0.285, 0.070, 0.42);
    let strike_tail = trailing_env(progress, 0.022, 7.5);
    let platen_tail = trailing_env(progress, 0.105, 8.5);
    let carriage_tail = trailing_env(progress, 0.285, 10.0);
    let impact_env = strike + platen + carriage * 0.80;

    let hard_click = high_pass_noise * spec.noise * (strike * 0.92 + platen * 0.46);
    let metal_tick = high_pass_noise.signum() * (strike * 0.22 + platen * 0.12);
    let clank_ring =
        (sin_hz(2_180.0, t) * 0.22 + sin_hz(3_120.0, t) * 0.34 + sin_hz(4_260.0, t) * 0.24)
            * spec.brightness
            * (strike_tail * 0.88 + platen_tail * 0.52);
    let metal_shimmer = high_pass_noise * spec.noise * (strike_tail * 0.24 + platen_tail * 0.16);
    let key_body = (sin_hz(spec.base_hz, t) * 0.22 + sin_hz(spec.base_hz * 1.62, t) * 0.14)
        * spec.body
        * impact_env;
    let carriage_knock =
        (sin_hz(620.0, t) * 0.12 + sin_hz(940.0, t) * 0.08) * spec.body * carriage_tail;

    (hard_click + metal_tick + clank_ring + metal_shimmer + key_body + carriage_knock) * release
}

fn sample_delete_snap(spec: &SoundSpec, t: f32, progress: f32, high_pass_noise: f32) -> f32 {
    let release = (1.0 - progress).max(0.0).powf(1.22);
    let click_env = (-progress * 90.0).exp();
    let snap_env = gaussian(progress, 0.075, 0.020);
    let frequency = spec.base_hz * (1.25 - progress * 0.42);
    let snap = (sin_hz(frequency, t) + sin_hz(frequency * 1.84, t) * 0.35) * snap_env;
    (high_pass_noise * spec.noise * click_env + snap * spec.brightness) * release
}

fn sample_space_bar(spec: &SoundSpec, t: f32, progress: f32, high_pass_noise: f32) -> f32 {
    let release = (1.0 - progress).max(0.0).powf(1.18);
    let thock = gaussian(progress, 0.035, 0.038);
    let stabilizer = gaussian(progress, 0.215, 0.055);
    let body = sin_hz(spec.base_hz, t) * thock * spec.body
        + sin_hz(spec.base_hz * 1.58, t) * stabilizer * spec.body * 0.58;
    let rattle = high_pass_noise * spec.noise * (thock + stabilizer * 0.72);
    (body + rattle) * release
}

fn impact(progress: f32, center: f32, width: f32, level: f32) -> f32 {
    if progress < center {
        0.0
    } else {
        gaussian(progress, center, width) * level
    }
}

fn trailing_env(progress: f32, start: f32, decay: f32) -> f32 {
    if progress < start {
        0.0
    } else {
        (-(progress - start) * decay).exp()
    }
}

fn gaussian(x: f32, center: f32, width: f32) -> f32 {
    let z = (x - center) / width.max(0.0001);
    (-z * z).exp()
}

fn sin_hz(frequency: f32, t: f32) -> f32 {
    (2.0 * PI * frequency * t).sin()
}

fn write_pcm16_mono_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_size as usize);

    wav.extend_from_slice(b"RIFF");
    push_u32(&mut wav, 36 + data_size);
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    push_u32(&mut wav, 16);
    push_u16(&mut wav, 1);
    push_u16(&mut wav, 1);
    push_u32(&mut wav, sample_rate);
    push_u32(&mut wav, sample_rate * 2);
    push_u16(&mut wav, 2);
    push_u16(&mut wav, 16);
    wav.extend_from_slice(b"data");
    push_u32(&mut wav, data_size);

    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn next_arg<I>(iter: &mut I, flag: &str) -> io::Result<String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .ok_or_else(|| invalid(format!("missing value for {flag}")))
}

fn parse_sample_rate(value: &str) -> io::Result<u32> {
    let sample_rate: u32 = value
        .parse()
        .map_err(|_| invalid(format!("invalid sample rate: {value}")))?;
    if !(8_000..=192_000).contains(&sample_rate) {
        return Err(invalid("sample rate must be between 8000 and 192000"));
    }
    Ok(sample_rate)
}

fn parse_volume(value: &str) -> io::Result<f32> {
    let parsed: f32 = value
        .parse()
        .map_err(|_| invalid(format!("invalid volume: {value}")))?;
    let volume = if parsed > 1.0 { parsed / 100.0 } else { parsed };
    if !(0.0..=1.0).contains(&volume) {
        return Err(invalid("volume must be 0.0-1.0 or 0-100"));
    }
    Ok(volume)
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn print_usage() {
    println!(
        "Usage: build_keyboard_sounds [--out wavs] [--preset soft|classic|crisp|blue|balanced|retro] [--volume 0.82] [--sample-rate 44100] [--trace-from your_own_wavs]"
    );
}

fn hash_name(name: &str) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_signed(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let value = (self.state >> 8) as f32 / 16_777_215.0;
        value * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_generates_fatmi_file_names() {
        let names: Vec<&str> = specs_for_preset(Preset::Classic)
            .iter()
            .map(|spec| spec.file_name)
            .collect();

        assert_eq!(
            names,
            vec![
                "0.wav",
                "1.wav",
                "2.wav",
                "3.wav",
                "4.wav",
                "5.wav",
                "6.wav",
                "7.wav",
                "8.wav",
                "9.wav",
                "enter.wav",
                "delete.wav",
                "backspace.wav",
                "space.wav",
            ]
        );
    }

    #[test]
    fn generated_wav_is_pcm16_mono_for_fatmi() {
        let spec = specs_for_preset(Preset::Classic)[0];
        let wav = build_wav_bytes(&spec, 44_100, 0.8, 1);

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1);
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1);
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            44_100
        );
        assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]) as usize,
            wav.len() - 44
        );
    }

    #[test]
    fn generated_sound_has_samples_and_is_short() {
        let spec = SoundSpec {
            file_name: "test.wav",
            kind: SoundKind::BlueKey,
            duration_ms: 40,
            base_hz: 1_600.0,
            noise: 0.9,
            body: 0.3,
            brightness: 1.0,
        };
        let wav = build_wav_bytes(&spec, 44_100, 0.8, 42);
        let pcm = &wav[44..];

        assert!(pcm.iter().any(|byte| *byte != 0));
        assert!(wav.len() < 8_000);
    }

    #[test]
    fn generated_sound_is_deterministic() {
        let spec = specs_for_preset(Preset::Crisp)[3];

        assert_eq!(
            build_wav_bytes(&spec, 44_100, 0.75, 99),
            build_wav_bytes(&spec, 44_100, 0.75, 99)
        );
    }

    #[test]
    fn parse_args_accepts_percentage_volume() {
        let options = parse_args([
            "--out".to_string(),
            "D:/tmp/fatmi-wavs".to_string(),
            "--preset".to_string(),
            "soft".to_string(),
            "--volume".to_string(),
            "65".to_string(),
            "--sample-rate".to_string(),
            "22050".to_string(),
        ])
        .unwrap();

        assert_eq!(options.out_dir, PathBuf::from("D:/tmp/fatmi-wavs"));
        assert_eq!(options.preset, PresetOption::Single(Preset::Soft));
        assert_eq!(options.sample_rate, 22_050);
        assert!((options.volume - 0.65).abs() < f32::EPSILON);
    }

    #[test]
    fn balanced_preset_has_longer_big_key_shape() {
        let specs = specs_for_preset(Preset::Balanced);
        let durations: Vec<(&str, u32)> = specs
            .iter()
            .map(|spec| (spec.file_name, spec.duration_ms))
            .collect();

        assert_eq!(
            durations,
            vec![
                ("0.wav", 122),
                ("1.wav", 197),
                ("2.wav", 139),
                ("3.wav", 156),
                ("4.wav", 111),
                ("5.wav", 199),
                ("6.wav", 134),
                ("7.wav", 137),
                ("8.wav", 172),
                ("9.wav", 97),
                ("enter.wav", 485),
                ("delete.wav", 175),
                ("backspace.wav", 175),
                ("space.wav", 168),
            ]
        );
    }

    #[test]
    fn parse_args_accepts_balanced_preset() {
        let options = parse_args(["--preset".to_string(), "balanced".to_string()]).unwrap();

        assert_eq!(options.preset, PresetOption::Single(Preset::Balanced));
    }

    #[test]
    fn parse_args_accepts_blue_preset() {
        let options = parse_args(["--preset".to_string(), "blue".to_string()]).unwrap();

        assert_eq!(options.preset, PresetOption::Single(Preset::Blue));
    }

    #[test]
    fn parse_args_accepts_retro_preset() {
        let options = parse_args(["--preset".to_string(), "retro".to_string()]).unwrap();

        assert_eq!(options.preset, PresetOption::Single(Preset::Retro));
    }

    #[test]
    fn parse_args_accepts_all_preset() {
        let options = parse_args(["--preset".to_string(), "all".to_string()]).unwrap();

        assert_eq!(options.preset, PresetOption::All);
    }

    #[test]
    fn retro_preset_uses_sweep_backspace_and_typewriter_enter() {
        let specs = specs_for_preset(Preset::Retro);
        let enter = specs
            .iter()
            .find(|spec| spec.file_name == "enter.wav")
            .unwrap();
        let backspace = specs
            .iter()
            .find(|spec| spec.file_name == "backspace.wav")
            .unwrap();
        assert_eq!(enter.kind, SoundKind::TypewriterEnter);
        assert_eq!(backspace.kind, SoundKind::RetroBackspace);
    }

    #[test]
    fn parse_args_accepts_trace_from_directory() {
        let options = parse_args([
            "--trace-from".to_string(),
            "D:/mytools/own_keyboard_recordings/wavs".to_string(),
        ])
        .unwrap();

        assert_eq!(
            options.trace_from,
            Some(PathBuf::from("D:/mytools/own_keyboard_recordings/wavs"))
        );
    }

    #[test]
    fn blue_preset_uses_realistic_switch_timing() {
        let specs = specs_for_preset(Preset::Blue);
        let key_specs: Vec<&SoundSpec> = specs
            .iter()
            .filter(|spec| spec.kind == SoundKind::BlueKey)
            .collect();

        assert!(key_specs
            .iter()
            .all(|spec| (78..=92).contains(&spec.duration_ms)));
        assert!(
            specs
                .iter()
                .find(|spec| spec.file_name == "space.wav")
                .unwrap()
                .duration_ms
                >= 100
        );
    }

    #[test]
    fn blue_key_has_clicky_front_transient() {
        let spec = specs_for_preset(Preset::Classic)[0];
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 7));

        let front = rms(&samples[0..ms_to_samples(44_100, 7)]);
        let middle = rms(&samples[ms_to_samples(44_100, 13)..ms_to_samples(44_100, 26)]);

        assert!(front > middle * 1.22, "front={front}, middle={middle}");
    }

    #[test]
    fn blue_preset_key_has_two_clicks_then_body() {
        let spec = specs_for_preset(Preset::Blue)[0];
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 7));
        let frame = ms_to_samples(44_100, 2);
        let values: Vec<f32> = samples
            .chunks(frame)
            .take(ms_to_samples(44_100, 28) / frame)
            .map(rms)
            .collect();
        let peak_count = values
            .windows(3)
            .filter(|triple| triple[1] > 0.11 && triple[1] > triple[0] && triple[1] > triple[2])
            .count();
        let front = rms(&samples[0..ms_to_samples(44_100, 18)]);
        let tail = rms(&samples[ms_to_samples(44_100, 35)..ms_to_samples(44_100, 70)]);

        assert!(
            peak_count >= 2,
            "values={values:?}, peak_count={peak_count}"
        );
        assert!(tail > front * 0.05, "front={front}, tail={tail}");
    }

    #[test]
    fn blue_preset_key_is_not_tonal_beep() {
        let spec = specs_for_preset(Preset::Blue)[0];
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 7));
        let segment = &samples[ms_to_samples(44_100, 18)..ms_to_samples(44_100, 76)];
        let tone_score = max_normalized_autocorrelation(
            segment,
            ms_to_samples(44_100, 1),
            ms_to_samples(44_100, 9),
        );

        assert!(tone_score < 0.45, "tone_score={tone_score}");
    }

    #[test]
    fn blue_preset_key_front_is_tight_not_slap() {
        let spec = specs_for_preset(Preset::Blue)[0];
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 7));
        let front = rms(&samples[0..ms_to_samples(44_100, 18)]);
        let late_tail = rms(&samples[ms_to_samples(44_100, 45)..ms_to_samples(44_100, 78)]);

        assert!(front < 0.18, "front={front}");
        assert!(
            late_tail < front * 0.22,
            "front={front}, late_tail={late_tail}"
        );
    }

    #[test]
    fn trace_wav_preserves_duration_without_copying_samples() {
        let reference_samples = vec![0, 1200, -2400, 3600, -4800, 2400, -1200, 0];
        let reference = write_pcm16_mono_wav(&reference_samples, 22_050);
        let parsed_ref = parse_pcm16_wav_mono(&reference).unwrap();
        let samples =
            synthesize_traced_samples(&parsed_ref.samples, parsed_ref.sample_rate, 0.8, 12);
        let traced = write_pcm16_mono_wav(&samples, parsed_ref.sample_rate);
        let parsed = parse_pcm16_wav_mono(&traced).unwrap();

        assert_eq!(parsed.sample_rate, 22_050);
        assert_eq!(parsed.samples.len(), reference_samples.len());
        assert_ne!(traced, reference);
        assert!(parsed.samples.iter().any(|sample| sample.abs() > 0.001));
    }

    #[test]
    fn trace_wav_follows_reference_envelope() {
        let mut reference_samples = vec![0_i16; 120];
        for sample in reference_samples.iter_mut().take(40).skip(8) {
            *sample = 9_000;
        }
        let reference = write_pcm16_mono_wav(&reference_samples, 44_100);
        let parsed_ref = parse_pcm16_wav_mono(&reference).unwrap();
        let samples =
            synthesize_traced_samples(&parsed_ref.samples, parsed_ref.sample_rate, 0.8, 21);
        let traced = write_pcm16_mono_wav(&samples, parsed_ref.sample_rate);
        let parsed = parse_pcm16_wav_mono(&traced).unwrap();
        let traced_samples: Vec<i16> = parsed
            .samples
            .iter()
            .map(|sample| (sample * i16::MAX as f32) as i16)
            .collect();

        let active = rms(&traced_samples[8..40]);
        let quiet = rms(&traced_samples[80..120]);
        assert!(active > quiet * 8.0, "active={active}, quiet={quiet}");
    }

    #[test]
    fn trace_wav_reduces_low_slap_material() {
        let mut reference_samples = vec![0_i16; 512];
        for sample in reference_samples.iter_mut().take(220).skip(20) {
            *sample = 10_000;
        }
        let reference = write_pcm16_mono_wav(&reference_samples, 44_100);
        let parsed_ref = parse_pcm16_wav_mono(&reference).unwrap();
        let samples =
            synthesize_traced_samples(&parsed_ref.samples, parsed_ref.sample_rate, 0.8, 33);
        let traced = write_pcm16_mono_wav(&samples, parsed_ref.sample_rate);
        let parsed = parse_pcm16_wav_mono(&traced).unwrap();
        let traced_samples: Vec<i16> = parsed
            .samples
            .iter()
            .map(|sample| (sample * i16::MAX as f32) as i16)
            .collect();

        let low_ratio = moving_average_low_ratio(&traced_samples, 16);
        assert!(low_ratio < 0.16, "low_ratio={low_ratio}");
    }

    #[test]
    fn trace_wav_is_clicks_not_sand() {
        let mut reference_samples = vec![0_i16; 2048];
        for sample in reference_samples.iter_mut().take(1500).skip(80) {
            *sample = 8_000;
        }
        let reference = write_pcm16_mono_wav(&reference_samples, 44_100);
        let parsed_ref = parse_pcm16_wav_mono(&reference).unwrap();
        let samples =
            synthesize_traced_samples(&parsed_ref.samples, parsed_ref.sample_rate, 0.8, 44);
        let traced = write_pcm16_mono_wav(&samples, parsed_ref.sample_rate);
        let parsed = parse_pcm16_wav_mono(&traced).unwrap();
        let traced_samples: Vec<i16> = parsed
            .samples
            .iter()
            .map(|sample| (sample * i16::MAX as f32) as i16)
            .collect();

        let zc_per_sec = zero_crossings(&traced_samples) as f32
            / (traced_samples.len() as f32 / parsed.sample_rate as f32);
        assert!(zc_per_sec < 12_000.0, "zc_per_sec={zc_per_sec}");
    }

    #[test]
    fn backspace_has_descending_whoosh() {
        let spec = specs_for_preset(Preset::Classic)
            .into_iter()
            .find(|spec| spec.file_name == "backspace.wav")
            .unwrap();
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 11));
        let third = samples.len() / 3;
        let early = zero_crossings(&samples[..third]);
        let late = zero_crossings(&samples[third * 2..]);

        assert!(early > late * 2, "early={early}, late={late}");
    }

    #[test]
    fn normal_keys_cover_zero_to_nine_with_varied_styles() {
        let specs: Vec<SoundSpec> = specs_for_preset(Preset::Classic)
            .into_iter()
            .filter(|spec| {
                matches!(
                    spec.file_name,
                    "0.wav"
                        | "1.wav"
                        | "2.wav"
                        | "3.wav"
                        | "4.wav"
                        | "5.wav"
                        | "6.wav"
                        | "7.wav"
                        | "8.wav"
                        | "9.wav"
                )
            })
            .collect();
        let names: Vec<&str> = specs.iter().map(|spec| spec.file_name).collect();
        let mut durations: Vec<u32> = specs.iter().map(|spec| spec.duration_ms).collect();

        durations.sort_unstable();
        durations.dedup();

        assert_eq!(
            names,
            vec![
                "0.wav", "1.wav", "2.wav", "3.wav", "4.wav", "5.wav", "6.wav", "7.wav", "8.wav",
                "9.wav"
            ]
        );
        assert!(durations.len() >= 8, "durations={durations:?}");
    }

    #[test]
    fn enter_has_typewriter_metal_clank() {
        let spec = specs_for_preset(Preset::Classic)
            .into_iter()
            .find(|spec| spec.file_name == "enter.wav")
            .unwrap();
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 13));
        let peaks = frame_rms_peaks(&samples, ms_to_samples(44_100, 6), 0.035);
        let front = rms(&samples[..ms_to_samples(44_100, 28)]);
        let ring = &samples[ms_to_samples(44_100, 28)..ms_to_samples(44_100, 95)];
        let ring_rms = rms(ring);
        let tail = rms(&samples[ms_to_samples(44_100, 105)..]);
        let ring_zc_per_sec = zero_crossings(ring) as f32 / (ring.len() as f32 / 44_100.0);

        assert!(peaks >= 2, "peaks={peaks}");
        assert!(ring_rms > front * 0.10, "front={front}, ring={ring_rms}");
        assert!(
            ring_zc_per_sec > 3_200.0,
            "ring_zc_per_sec={ring_zc_per_sec}"
        );
        assert!(tail < front * 0.36, "front={front}, tail={tail}");
    }

    #[test]
    fn trace_mode_keeps_enter_synthesized_but_traces_backspace() {
        let specs = specs_for_preset(Preset::Classic);
        let zero = specs.iter().find(|spec| spec.file_name == "0.wav").unwrap();
        let enter = specs
            .iter()
            .find(|spec| spec.file_name == "enter.wav")
            .unwrap();
        let backspace = specs
            .iter()
            .find(|spec| spec.file_name == "backspace.wav")
            .unwrap();

        assert!(should_trace_reference(zero));
        assert!(should_trace_reference(enter));
        assert!(should_trace_reference(backspace));
    }

    #[test]
    fn balanced_enter_has_big_key_body() {
        let spec = specs_for_preset(Preset::Balanced)
            .into_iter()
            .find(|spec| spec.file_name == "enter.wav")
            .unwrap();
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 13));
        let front_len = ms_to_samples(44_100, 160).min(samples.len());

        assert!(rms(&samples[..front_len]) > 0.080);
    }

    #[test]
    fn balanced_backspace_whoosh_is_not_hash_noise() {
        let spec = specs_for_preset(Preset::Balanced)
            .into_iter()
            .find(|spec| spec.file_name == "backspace.wav")
            .unwrap();
        let samples = samples_from_wav(&build_wav_bytes(&spec, 44_100, 0.82, 11));
        let zc_per_sec = zero_crossings(&samples) as f32 / (samples.len() as f32 / 44_100.0);

        assert!(zc_per_sec < 4_500.0, "zc_per_sec={zc_per_sec}");
    }

    fn samples_from_wav(wav: &[u8]) -> Vec<i16> {
        wav[44..]
            .chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
            .collect()
    }

    fn ms_to_samples(sample_rate: u32, ms: u32) -> usize {
        ((sample_rate as f32) * (ms as f32 / 1000.0)).round() as usize
    }

    fn rms(samples: &[i16]) -> f32 {
        let sum: f32 = samples
            .iter()
            .map(|sample| {
                let value = *sample as f32 / i16::MAX as f32;
                value * value
            })
            .sum();
        (sum / samples.len().max(1) as f32).sqrt()
    }

    fn zero_crossings(samples: &[i16]) -> usize {
        samples
            .windows(2)
            .filter(|pair| (pair[0] < 0 && pair[1] >= 0) || (pair[0] >= 0 && pair[1] < 0))
            .count()
    }

    fn frame_rms_peaks(samples: &[i16], frame_size: usize, threshold: f32) -> usize {
        let values: Vec<f32> = samples.chunks(frame_size.max(1)).map(rms).collect();

        if values.is_empty() {
            return 0;
        }

        let mut count = 0;
        if values.len() == 1 {
            return usize::from(values[0] > threshold);
        }
        if values[0] > threshold && values[0] > values[1] {
            count += 1;
        }

        count += values
            .windows(3)
            .filter(|triple| {
                triple[1] > threshold && triple[1] > triple[0] && triple[1] > triple[2]
            })
            .count();

        if values[values.len() - 1] > threshold
            && values[values.len() - 1] > values[values.len() - 2]
        {
            count += 1;
        }

        count
    }

    fn moving_average_low_ratio(samples: &[i16], window: usize) -> f32 {
        let total = rms(samples);
        if total <= f32::EPSILON || window == 0 {
            return 0.0;
        }

        let mut smoothed = Vec::with_capacity(samples.len());
        let mut queue = std::collections::VecDeque::new();
        let mut sum = 0.0_f32;
        for sample in samples {
            let value = *sample as f32;
            queue.push_back(value);
            sum += value;
            if queue.len() > window {
                if let Some(old) = queue.pop_front() {
                    sum -= old;
                }
            }
            smoothed.push((sum / queue.len() as f32) as i16);
        }

        rms(&smoothed) / total
    }

    fn max_normalized_autocorrelation(samples: &[i16], min_lag: usize, max_lag: usize) -> f32 {
        let values: Vec<f32> = samples
            .iter()
            .map(|sample| *sample as f32 / i16::MAX as f32)
            .collect();
        let energy: f32 = values.iter().map(|sample| sample * sample).sum();
        if energy <= f32::EPSILON {
            return 0.0;
        }

        let mut max_score = 0.0_f32;
        for lag in min_lag..=max_lag.min(values.len().saturating_sub(1)) {
            let mut sum = 0.0_f32;
            for i in 0..(values.len() - lag) {
                sum += values[i] * values[i + lag];
            }
            max_score = max_score.max((sum / energy).abs());
        }
        max_score
    }
}
