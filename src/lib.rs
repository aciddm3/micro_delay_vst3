use nih_plug::plugin::vst3::Vst3Plugin;
use nih_plug::prelude::*;
use nih_plug::wrapper::vst3::subcategories::Vst3SubCategory;
use nih_plug_egui::{EguiState, create_egui_editor, egui, widgets};
use std::sync::Arc;

mod utils;

#[derive(Params)]
struct DParams {
    #[id = "dry_level"]
    pub dry: FloatParam,
    #[id = "wet_level"]
    pub wet1: FloatParam,
    #[id = "inverse_wet_level"]
    pub inverse_wet1: BoolParam,
    #[id = "delay"]
    pub delay1: FloatParam,
    #[id = "feedback"]
    pub fb1: FloatParam,
    #[id = "inverse_feedback"]
    pub inverse_fb1: BoolParam,
}

const MIN_GAIN: f32 = -80.0; // dB
const MAX_LINE_GAIN: f32 = 20.0;
const MAX_FEEDBACK_GAIN: f32 = 0.0;
const MIN_DELAY_TIME: f32 = 25.0; // microseconds
const MAX_DELAY_TIME: f32 = 100_000.0;

impl Default for DParams {
    fn default() -> Self {
        Self {
            dry: FloatParam::new(
                "Dry",
                0.0,
                FloatRange::Linear {
                    min: MIN_GAIN,
                    max: MAX_LINE_GAIN,
                },
            )
            .with_value_to_string(Arc::new(|s| {
                format!(
                    "{:2.2}dB = {:5.2}%",
                    if s < -100.0 { -std::f32::INFINITY } else { s },
                    utils::db_to_percent_gain(s)
                )
            })),
            wet1: FloatParam::new(
                "Line gain",
                MIN_GAIN,
                FloatRange::Linear {
                    min: MIN_GAIN,
                    max: MAX_LINE_GAIN,
                },
            )
            .with_value_to_string(Arc::new(|s| {
                format!(
                    "{:2.2}dB = {:5.2}%",
                    if s < -100.0 { -std::f32::INFINITY } else { s },
                    utils::db_to_percent_gain(s)
                )
            })),
            inverse_wet1: BoolParam::new("Line signal inverse", false),
            delay1: FloatParam::new(
                "Delay",
                MAX_DELAY_TIME,
                FloatRange::Linear {
                    min: MIN_DELAY_TIME,
                    max: MAX_DELAY_TIME,
                },
            )
            .with_value_to_string(Arc::new(|s| format!("{s:.0} microsec"))),
            fb1: FloatParam::new(
                "Feedback",
                MIN_GAIN,
                FloatRange::Linear {
                    min: MIN_GAIN,
                    max: MAX_FEEDBACK_GAIN,
                },
            )
            .with_value_to_string(Arc::new(|s| {
                format!(
                    "{:2.2}dB={:5.2}%",
                    if s < -100.0 { -std::f32::INFINITY } else { s },
                    utils::db_to_percent_gain(s)
                )
            })),
            inverse_fb1: BoolParam::new("Feedback inverse", false),
        }
    }
}
struct Delay {
    params: Arc<DParams>,
    samplerate: f32,
    channel_buffer: Vec<Vec<f32>>,
    buffer_size: usize,
    current_arrow_pos: Vec<usize>,

    dry_automation_samples: Vec<f32>,
    wet1_automation_samples: Vec<f32>,
    feedback1_automation_samples: Vec<f32>,
    delay1_automation_samples: Vec<f32>,

    editor_state: Arc<EguiState>,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            params: Default::default(),
            samplerate: Default::default(),
            channel_buffer: Default::default(),
            buffer_size: Default::default(),
            current_arrow_pos: Default::default(),
            dry_automation_samples: Default::default(),
            wet1_automation_samples: Default::default(),
            feedback1_automation_samples: Default::default(),
            delay1_automation_samples: Default::default(),
            editor_state: EguiState::from_size(640, 480),
        }
    }
}

impl Plugin for Delay {
    type SysExMessage = ();
    type BackgroundTask = ();

    const NAME: &'static str = "MicroDelay";
    const VENDOR: &'static str = "Gema";
    const URL: &'static str = "https://example.com/micro-delay";
    const EMAIL: &'static str = "None";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.samplerate = buffer_config.sample_rate;
        let num_channels = audio_io_layout
            .main_output_channels
            .map(|n| n.get())
            .unwrap_or(0);

        self.buffer_size = (self.samplerate * MAX_DELAY_TIME / 100_000.0) as usize + 5;
        self.channel_buffer
            .resize(num_channels as usize, vec![0.0; self.buffer_size]);
        // Буфер на MAX_DELAY_TIME
        // + 5 сэмплов на всякий случай

        self.current_arrow_pos.resize(num_channels as usize, 0);
        // инициализируем позиции кареток

        self.dry_automation_samples = vec![0.0; buffer_config.max_buffer_size as usize];
        self.wet1_automation_samples = vec![0.0; buffer_config.max_buffer_size as usize];
        self.feedback1_automation_samples = vec![0.0; buffer_config.max_buffer_size as usize];
        self.delay1_automation_samples = vec![0.0; buffer_config.max_buffer_size as usize];
        true
    }

    fn reset(&mut self) {
        self.channel_buffer.iter_mut().for_each(|s| s.fill(0.0));
        self.current_arrow_pos.fill(0);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let samples_per_buffer = buffer.samples();

        let dry_samples = &mut self.dry_automation_samples;
        self.params
            .dry
            .smoothed
            .next_block(dry_samples, samples_per_buffer);
        dry_samples
            .iter_mut()
            .for_each(|s| *s = utils::db_to_gain(*s));

        let wet1_samples = &mut self.wet1_automation_samples;
        self.params
            .wet1
            .smoothed
            .next_block(wet1_samples, samples_per_buffer);
        wet1_samples
            .iter_mut()
            .for_each(|s| *s = utils::db_to_gain(*s));

        let feedback1_samples = &mut self.feedback1_automation_samples;
        self.params
            .fb1
            .smoothed
            .next_block(feedback1_samples, samples_per_buffer);
        feedback1_samples
            .iter_mut()
            .for_each(|s| *s = utils::db_to_gain(*s));

        let delay1_samples = &mut self.delay1_automation_samples;
        self.params
            .delay1
            .smoothed
            .next_block(delay1_samples, samples_per_buffer);

        for (channel_idx, samples) in buffer.as_slice().iter_mut().enumerate() {
            let arrow_pos = &mut self.current_arrow_pos[channel_idx];
            let current_delay_buffer = &mut self.channel_buffer[channel_idx];
            for (sample_idx, sample) in samples.iter_mut().enumerate() {
                let delay_time_in_samples_f = self.samplerate * delay1_samples[sample_idx] / 1e6;
                let delay_time_whole_samples = delay_time_in_samples_f.ceil() as usize;
                let interpolation_ratio = delay_time_in_samples_f.fract();

                let value_to_play = utils::convex(
                    current_delay_buffer[(*arrow_pos as isize - delay_time_whole_samples as isize)
                        .rem_euclid(self.buffer_size as isize)
                        as usize],
                    current_delay_buffer[(*arrow_pos as isize - delay_time_whole_samples as isize
                        + 1)
                    .rem_euclid(self.buffer_size as isize)
                        as usize],
                    interpolation_ratio,
                );

                let feedback = value_to_play
                    * feedback1_samples[sample_idx]
                    * utils::factor_sign(self.params.inverse_fb1.value());

                current_delay_buffer[*arrow_pos] = *sample + feedback;

                // Вычисление компонент
                let dry_component = *sample * dry_samples[sample_idx];
                let wet_component = value_to_play
                    * wet1_samples[sample_idx]
                    * utils::factor_sign(self.params.inverse_wet1.value());

                // Смешивание
                *sample = dry_component + wet_component;

                // сдвиг каретки
                *arrow_pos += 1;
                // возврат каретки
                if *arrow_pos >= self.buffer_size {
                    *arrow_pos = 0;
                }
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();

        create_egui_editor(
            self.editor_state.clone(),
            (),               // Данные для синхронизации, если нужны
            |_ctx, _data| {}, // Инициализация
            move |egui_ctx, setter, _data| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("MicroDelay");
                    ui.separator();

                    // Стандартный слайдер для Dry
                    ui.label("Dry Level");
                    ui.add(widgets::ParamSlider::for_param(&params.dry, setter));

                    ui.separator();

                    ui.label("Wet Level");
                    ui.add(widgets::ParamSlider::for_param(&params.wet1, setter));
                    ui.label("Inverse Wet");
                    ui.add(widgets::ParamSlider::for_param(
                        &params.inverse_wet1,
                        setter,
                    ));

                    ui.separator();

                    ui.label("Delay (microsec)");
                    ui.add(widgets::ParamSlider::for_param(&params.delay1, setter));

                    ui.separator();

                    ui.label("Feedback");
                    ui.add(widgets::ParamSlider::for_param(&params.fb1, setter));
                    ui.label("Inverse feedback");
                    ui.add(widgets::ParamSlider::for_param(&params.inverse_fb1, setter));

                    // Чекбоксы для инверсии
                });
            },
        )
    }
}

impl Vst3Plugin for Delay {
    const VST3_CLASS_ID: [u8; 16] = [
        98, 218, 94, 45, 78, 214, 174, 224, 167, 126, 143, 79, 37, 188, 235, 30,
    ]; // UUID is generated randomly
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Delay];
}

nih_export_vst3!(Delay);
