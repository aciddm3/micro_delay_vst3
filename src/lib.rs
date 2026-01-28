use nih_plug::plugin::vst3::Vst3Plugin;
use nih_plug::prelude::*;
use nih_plug::wrapper::vst3::subcategories::Vst3SubCategory;
use nih_plug_egui::{EguiState, create_egui_editor, egui, widgets};
use std::sync::Arc;

mod delay_line;
mod utils;

#[derive(Params)]
struct DParams {
    #[id = "in_send_out"]
    pub in_send_out: FloatParam,
    #[id = "in_send_a"]
    pub in_send_a: FloatParam,
    #[id = "in_send_b"]
    pub in_send_b: FloatParam,

    #[id = "line_a_level"]
    pub a_send_out: FloatParam,
    #[id = "line_a_delay"]
    pub delay_a: FloatParam,
    #[id = "line_a_feedback"]
    pub fb_a: FloatParam,
    #[id = "a_to_b_send"]
    pub a_send_b: FloatParam,

    #[id = "line_b_level"]
    pub b_send_out: FloatParam,
    #[id = "line_b_delay"]
    pub delay_b: FloatParam,
    #[id = "line_b_feedback"]
    pub fb_b: FloatParam,
    #[id = "b_to_a_send"]
    pub b_send_a: FloatParam,

    #[id = "in_to_A_pan"]
    pub in_send_a_pan: FloatParam,
    #[id = "A_to_out_pan"]
    pub a_send_out_pan: FloatParam,
    #[id = "in_to_B_pan"]
    pub in_send_b_pan: FloatParam,
    #[id = "B_to_out_pan"]
    pub b_send_out_pan: FloatParam,
}

const MIN_DELAY_TIME: f32 = 0.025; // milliseconds
const MAX_DELAY_TIME: f32 = 16000.0;

impl Default for DParams {
    fn default() -> Self {
        Self {
            in_send_out: FloatParam::new(
                "Dry Level",
                100.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            in_send_a: FloatParam::new(
                "Input to A",
                100.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            in_send_b: FloatParam::new(
                "Input to B",
                100.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            a_send_out: FloatParam::new(
                "A to out",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            b_send_out: FloatParam::new(
                "B to out",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            a_send_b: FloatParam::new(
                "A to B",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            b_send_a: FloatParam::new(
                "B to A",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),

            fb_a: FloatParam::new(
                "Feedback A",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),
            fb_b: FloatParam::new(
                "Feedback B",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit("%"),

            delay_a: FloatParam::new(
                "Delay A",
                500.0,
                FloatRange::Linear {
                    min: MIN_DELAY_TIME,
                    max: MAX_DELAY_TIME,
                },
            )
            .with_unit("ms"),
            delay_b: FloatParam::new(
                "Delay B",
                500.0,
                FloatRange::Linear {
                    min: MIN_DELAY_TIME,
                    max: MAX_DELAY_TIME,
                },
            )
            .with_unit("ms"),
            in_send_a_pan: FloatParam::new(
                "In to A pan",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_value_to_string(Arc::new(|s| format!("{s:.2}"))),
            a_send_out_pan: FloatParam::new(
                "A to out pan",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_value_to_string(Arc::new(|s| format!("{s:.2}"))),
            in_send_b_pan: FloatParam::new(
                "In to B pan",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_value_to_string(Arc::new(|s| format!("{s:.2}"))),
            b_send_out_pan: FloatParam::new(
                "B to out pan",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_value_to_string(Arc::new(|s| format!("{s:.2}"))),
        }
    }
}
struct Delay {
    params: Arc<DParams>,
    samplerate: f32,

    line_a: delay_line::DelayLine,
    line_b: delay_line::DelayLine,

    in_send_a_automation_samples: Vec<f32>,
    in_send_b_automation_samples: Vec<f32>,

    a_send_out_automation_samples: Vec<f32>,
    b_send_out_automation_samples: Vec<f32>,
    dry_automation_samples: Vec<f32>,

    a_send_b_automation_samples: Vec<f32>,
    b_send_a_automation_samples: Vec<f32>,

    in_send_a_pan_automation_samples: Vec<f32>,
    a_send_out_pan_automation_samples: Vec<f32>,
    in_send_b_pan_automation_samples: Vec<f32>,
    b_send_out_pan_automation_samples: Vec<f32>,

    editor_state: Arc<EguiState>,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            params: Default::default(),
            samplerate: Default::default(),

            in_send_a_automation_samples: Default::default(),
            in_send_b_automation_samples: Default::default(),
            a_send_out_automation_samples: Default::default(),
            b_send_out_automation_samples: Default::default(),
            dry_automation_samples: Default::default(),

            a_send_b_automation_samples: Default::default(),
            b_send_a_automation_samples: Default::default(),

            in_send_a_pan_automation_samples: Default::default(),
            a_send_out_pan_automation_samples: Default::default(),
            in_send_b_pan_automation_samples: Default::default(),
            b_send_out_pan_automation_samples: Default::default(),

            line_a: Default::default(),
            line_b: Default::default(),

            editor_state: EguiState::from_size(740, 475),
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

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

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

        self.line_a.init(
            (self.samplerate * MAX_DELAY_TIME / 1e3) as usize + 5,
            num_channels as usize,
            buffer_config.max_buffer_size as usize,
            self.samplerate,
        );

        self.line_b.init(
            (self.samplerate * MAX_DELAY_TIME / 1e3) as usize + 5,
            num_channels as usize,
            buffer_config.max_buffer_size as usize,
            self.samplerate,
        );

        let max_buffer_size = buffer_config.max_buffer_size as usize;

        self.in_send_a_automation_samples = vec![0.0; max_buffer_size];
        self.in_send_b_automation_samples = vec![0.0; max_buffer_size];
        self.a_send_out_automation_samples = vec![0.0; max_buffer_size];
        self.b_send_out_automation_samples = vec![0.0; max_buffer_size];
        self.dry_automation_samples = vec![0.0; max_buffer_size];
        self.a_send_b_automation_samples = vec![0.0; max_buffer_size];
        self.b_send_a_automation_samples = vec![0.0; max_buffer_size];

        self.in_send_a_pan_automation_samples = vec![0.0; max_buffer_size];
        self.in_send_b_pan_automation_samples = vec![0.0; max_buffer_size];
        self.b_send_out_pan_automation_samples = vec![0.0; max_buffer_size];
        self.a_send_out_pan_automation_samples = vec![0.0; max_buffer_size];

        true
    }

    fn reset(&mut self) {
        self.line_a.reset();
        self.line_b.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let block_len = buffer.samples();

        // заполнение автоматизации
        {
            self.params
                .a_send_b
                .smoothed
                .next_block(&mut self.a_send_b_automation_samples, block_len);
            self.params
                .a_send_out
                .smoothed
                .next_block(&mut self.a_send_out_automation_samples, block_len);
            self.params
                .b_send_a
                .smoothed
                .next_block(&mut self.b_send_a_automation_samples, block_len);
            self.params
                .b_send_out
                .smoothed
                .next_block(&mut self.b_send_out_automation_samples, block_len);
            self.params
                .delay_a
                .smoothed
                .next_block(&mut self.line_a.delay_automation_samples, block_len);
            self.params
                .delay_b
                .smoothed
                .next_block(&mut self.line_b.delay_automation_samples, block_len);
            self.params
                .fb_a
                .smoothed
                .next_block(&mut self.line_a.feedback_automation_samples, block_len);
            self.params
                .fb_b
                .smoothed
                .next_block(&mut self.line_b.feedback_automation_samples, block_len);
            self.params
                .in_send_a
                .smoothed
                .next_block(&mut self.in_send_a_automation_samples, block_len);
            self.params
                .in_send_b
                .smoothed
                .next_block(&mut self.in_send_b_automation_samples, block_len);
            self.params
                .in_send_out
                .smoothed
                .next_block(&mut self.dry_automation_samples, block_len);

            self.params
                .a_send_out_pan
                .smoothed
                .next_block(&mut self.a_send_out_pan_automation_samples, block_len);
            self.params
                .b_send_out_pan
                .smoothed
                .next_block(&mut self.b_send_out_pan_automation_samples, block_len);

            self.params
                .in_send_a_pan
                .smoothed
                .next_block(&mut self.in_send_a_pan_automation_samples, block_len);
            self.params
                .in_send_b_pan
                .smoothed
                .next_block(&mut self.in_send_b_pan_automation_samples, block_len);
        }

        {
            self.a_send_b_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.a_send_out_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.b_send_a_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.b_send_out_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.line_a
                .feedback_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.line_b
                .feedback_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.in_send_a_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.in_send_b_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));
            self.dry_automation_samples
                .iter_mut()
                .for_each(|s| *s = utils::knob_gain(*s));

            self.line_a
                .delay_automation_samples
                .iter_mut()
                .for_each(|s| *s = self.samplerate * *s / 1e3);
            self.line_b
                .delay_automation_samples
                .iter_mut()
                .for_each(|s| *s = self.samplerate * *s / 1e3);
        }

        for (channel_idx, samples) in buffer.as_slice().iter_mut().enumerate() {
            for (sample_idx, sample) in samples.iter_mut().enumerate() {
                let (in_send_a_pan, in_send_b_pan, a_send_out_pan, b_send_out_pan) =
                    if channel_idx % 2 == 0 {
                        //left
                        (
                            utils::balance_ratio_to_stereo_coefficients(
                                self.in_send_a_pan_automation_samples[sample_idx],
                            )
                            .0,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.in_send_b_pan_automation_samples[sample_idx],
                            )
                            .0,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.a_send_out_pan_automation_samples[sample_idx],
                            )
                            .0,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.b_send_out_pan_automation_samples[sample_idx],
                            )
                            .0,
                        )
                    } else {
                        //right
                        (
                            utils::balance_ratio_to_stereo_coefficients(
                                self.in_send_a_pan_automation_samples[sample_idx],
                            )
                            .1,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.in_send_b_pan_automation_samples[sample_idx],
                            )
                            .1,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.a_send_out_pan_automation_samples[sample_idx],
                            )
                            .1,
                            utils::balance_ratio_to_stereo_coefficients(
                                self.b_send_out_pan_automation_samples[sample_idx],
                            )
                            .1,
                        )
                    };

                self.line_a
                    .set_delay(self.line_a.delay_automation_samples[sample_idx]);
                self.line_b
                    .set_delay(self.line_b.delay_automation_samples[sample_idx]);

                let value_to_play_a = self.line_a.read_value_from_channel(channel_idx);
                let value_to_play_b = self.line_b.read_value_from_channel(channel_idx);

                // Внутри цикла по сэмплам:
                self.line_a.write_value_to_channel(
                    // Умножаем входной сигнал на параметр посыла
                    (*sample * self.in_send_a_automation_samples[sample_idx] * in_send_a_pan)
                        + (value_to_play_b * self.b_send_a_automation_samples[sample_idx])
                        + (value_to_play_a * self.line_a.feedback_automation_samples[sample_idx]),
                    channel_idx,
                );

                self.line_b.write_value_to_channel(
                    // Аналогично для линии B
                    (*sample * self.in_send_b_automation_samples[sample_idx] * in_send_b_pan)
                        + (value_to_play_a * self.a_send_b_automation_samples[sample_idx])
                        + (value_to_play_b * self.line_b.feedback_automation_samples[sample_idx]),
                    channel_idx,
                );

                // Вычисление компонент
                let dry_component = *sample * self.dry_automation_samples[sample_idx];
                let wet_component = value_to_play_a * self.a_send_out_automation_samples[sample_idx] * a_send_out_pan
                    + value_to_play_b * self.b_send_out_automation_samples[sample_idx] * b_send_out_pan;

                // Смешивание
                *sample = dry_component + wet_component;

                // сдвиг каретки
                self.line_a.move_arrow_over_channel(channel_idx);
                self.line_b.move_arrow_over_channel(channel_idx);
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();

        const COLOR_A: egui::Color32 = egui::Color32::from_rgb(150, 255, 0); // Ярко-зеленый
        const COLOR_B: egui::Color32 = egui::Color32::from_rgb(150, 0, 255); // Фиолетовый
        const COLOR_DRY: egui::Color32 = egui::Color32::from_rgb(0, 255, 255); // Циан (центр)

        create_egui_editor(
            self.editor_state.clone(),
            (),
            |_ctx, _data| {},
            move |egui_ctx, setter, _data| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("MICRODELAY MATRIX").strong().size(20.0));
                    });
                    ui.add_space(15.0);

                    // Используем сетку, чтобы повторить топологию Delay.png
                    egui::Grid::new("delay_matrix_grid")
                        .spacing([60.0, 5.0])
                        .min_col_width(120.0)
                        .show(ui, |ui| {
                            // --- РЯД 1: Входные посылы (Верхние крутилки на схеме) ---
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("INPUT -> A").color(COLOR_A));
                                ui.add(widgets::ParamSlider::for_param(&params.in_send_a, setter));
                                ui.label(egui::RichText::new("INPUT -> A \n BAL").color(COLOR_A));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.in_send_a_pan,
                                    setter,
                                ));
                            });

                            // Пустое место над Dry
                            ui.label("");

                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("INPUT -> B").color(COLOR_B));
                                ui.add(widgets::ParamSlider::for_param(&params.in_send_b, setter));
                                ui.label(egui::RichText::new("INPUT -> B \n BAL").color(COLOR_B));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.in_send_b_pan,
                                    setter,
                                ));
                            });
                            ui.end_row();

                            // --- РЯД 2: Кросс-фидбек A -> B (Верхняя горизонтальная линия) ---
                            ui.label("");
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("A -> B").color(COLOR_A));
                                ui.add(widgets::ParamSlider::for_param(&params.a_send_b, setter));
                            });
                            ui.label("");
                            ui.end_row();

                            // --- РЯД 3: Основные блоки задержки и Dry (Центр схемы) ---
                            // Слева: Блок A
                            ui.vertical_centered(|ui| {
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("LINE A").strong().color(COLOR_A));
                                    ui.label("Time (ms)");
                                    ui.add(widgets::ParamSlider::for_param(
                                        &params.delay_a,
                                        setter,
                                    ));
                                    ui.label("Local FB");
                                    ui.add(widgets::ParamSlider::for_param(&params.fb_a, setter));
                                });
                            });

                            // В центре: Dry Level
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("IN -> OUT").color(COLOR_DRY));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.in_send_out,
                                    setter,
                                ));
                            });

                            // Справа: Блок B
                            ui.vertical_centered(|ui| {
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("LINE B").strong().color(COLOR_B));
                                    ui.label("Time (ms)");
                                    ui.add(widgets::ParamSlider::for_param(
                                        &params.delay_b,
                                        setter,
                                    ));
                                    ui.label("Local FB");
                                    ui.add(widgets::ParamSlider::for_param(&params.fb_b, setter));
                                });
                            });
                            ui.end_row();

                            // --- РЯД 4: Кросс-фидбек B -> A (Нижняя горизонтальная линия) ---
                            ui.label("");
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("B -> A").color(COLOR_B));
                                ui.add(widgets::ParamSlider::for_param(&params.b_send_a, setter));
                            });
                            ui.label("");
                            ui.end_row();

                            // --- РЯД 5: Выходы в мастер (Нижние крутилки на схеме) ---
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("A -> OUT").color(COLOR_A));
                                ui.add(widgets::ParamSlider::for_param(&params.a_send_out, setter));
                                ui.label(egui::RichText::new("A -> OUT \n BAL").color(COLOR_A));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.a_send_out_pan,
                                    setter,
                                ));
                            });

                            ui.label(""); // Точка суммирования

                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("B -> OUT").color(COLOR_B));
                                ui.add(widgets::ParamSlider::for_param(&params.b_send_out, setter));
                                ui.label(egui::RichText::new("B -> OUT \n BAL").color(COLOR_B));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.b_send_out_pan,
                                    setter,
                                ));
                            });
                            ui.end_row();
                        });
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
