pub struct PlaybackState {
    pub current_frame: usize,
    pub total_frames: usize,
    pub is_playing: bool,
    pub playback_speed: f32,
    accumulated_time: f32,
}

impl PlaybackState {
    pub fn new(total_frames: usize) -> Self {
        Self {
            current_frame: 0,
            total_frames,
            is_playing: false,
            playback_speed: 1.0,
            accumulated_time: 0.0,
        }
    }

    pub fn tick(&mut self, delta_seconds: f32, timestamps: &[f64]) {
        if !self.is_playing || self.total_frames == 0 {
            return;
        }
        self.accumulated_time += delta_seconds * self.playback_speed;

        if self.current_frame + 1 < timestamps.len() {
            let next_ts = timestamps[self.current_frame + 1] as f32;
            let curr_ts = timestamps[self.current_frame] as f32;
            let frame_dur = (next_ts - curr_ts).max(0.001);
            if self.accumulated_time >= frame_dur {
                self.accumulated_time -= frame_dur;
                self.current_frame += 1;
                if self.current_frame + 1 >= self.total_frames {
                    self.current_frame = self.total_frames.saturating_sub(1);
                    self.is_playing = false;
                }
            }
        } else {
            self.is_playing = false;
        }
    }

    pub fn seek(&mut self, frame: usize) {
        self.current_frame = frame.min(self.total_frames.saturating_sub(1));
        self.accumulated_time = 0.0;
    }

    pub fn toggle_play(&mut self) {
        if self.current_frame + 1 >= self.total_frames {
            self.current_frame = 0;
        }
        self.is_playing = !self.is_playing;
        self.accumulated_time = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_advances_frame() {
        let timestamps: Vec<f64> = (0..10).map(|i| i as f64 * 0.033).collect();
        let mut state = PlaybackState::new(timestamps.len());
        state.is_playing = true;
        state.tick(0.1, &timestamps);
        assert!(state.current_frame > 0);
    }

    #[test]
    fn test_seek_clamps() {
        let mut state = PlaybackState::new(10);
        state.seek(999);
        assert_eq!(state.current_frame, 9);
    }

    #[test]
    fn test_speed_control() {
        let timestamps: Vec<f64> = (0..100).map(|i| i as f64 * 0.033).collect();
        let mut s1 = PlaybackState::new(timestamps.len());
        let mut s2 = PlaybackState::new(timestamps.len());
        s1.is_playing = true;
        s2.is_playing = true;
        s2.playback_speed = 2.0;
        s1.tick(0.1, &timestamps);
        s2.tick(0.1, &timestamps);
        assert!(s2.current_frame >= s1.current_frame);
    }
}
