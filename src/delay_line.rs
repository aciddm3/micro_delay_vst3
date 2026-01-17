#[derive(Debug, Default)]
pub struct DelayLine {
    pub delay: f32,
    pub samplerate: f32,
    pub channel_delay_buffer: Vec<Vec<f32>>,
    pub delay_buffer_size: usize,
    pub current_arrow_pos: Vec<isize>,

    pub feedback_automation_samples: Vec<f32>,

    pub delay_automation_samples: Vec<f32>,
}

impl DelayLine {
    pub fn init(
        &mut self,
        delay_buffer_size: usize,
        channels_number: usize,
        max_buffer_size: usize,
        samplerate: f32,
    ) {
        self.delay = 0.0;
        self.samplerate = samplerate;
        self.channel_delay_buffer = vec![vec![0.0; delay_buffer_size]; channels_number];
        self.delay_buffer_size = delay_buffer_size;
        self.current_arrow_pos = vec![0; channels_number];
        self.feedback_automation_samples = vec![0.0; max_buffer_size];
        self.delay_automation_samples = vec![0.0; max_buffer_size];
    }

    pub fn read_value_from_channel(&mut self, channel_idx: usize) -> f32 {
        let arrow_pos = &mut self.current_arrow_pos[channel_idx];
        let current_delay_buffer = &mut self.channel_delay_buffer[channel_idx];

        let delay_time_whole_samples = self.delay.ceil() as isize;
        let interpolation_ratio = self.delay.fract();

        let mut idx_a = *arrow_pos - delay_time_whole_samples;
        while idx_a < 0 {
            idx_a += self.delay_buffer_size as isize;
        }

        let mut idx_b = idx_a + 1;
        if idx_b >= self.delay_buffer_size as isize {
            idx_b = 0;
        }

        crate::utils::convex(
            current_delay_buffer[idx_a as usize],
            current_delay_buffer[idx_b as usize],
            interpolation_ratio,
        )
    }

    pub fn write_value_to_channel(&mut self, value_to_write: f32, channel_idx: usize) {
        let arrow_pos = &mut self.current_arrow_pos[channel_idx];
        let current_delay_buffer = &mut self.channel_delay_buffer[channel_idx];

        current_delay_buffer[*arrow_pos as usize] = value_to_write;
    }

    pub fn move_arrow_over_channel(&mut self, channel_idx: usize) {
        if self.current_arrow_pos[channel_idx] >= self.delay_buffer_size as isize - 1 {
            self.current_arrow_pos[channel_idx] = 0
        } else {
            self.current_arrow_pos[channel_idx] += 1
        }
    }

    pub fn set_delay(&mut self, delay_in_float_samples: f32) {
        self.delay = delay_in_float_samples;
    }

    pub fn reset(&mut self) {
        self.channel_delay_buffer
            .iter_mut()
            .for_each(|s| s.fill(0.0));
        self.current_arrow_pos.fill(0);
    }
}
