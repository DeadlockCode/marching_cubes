pub enum TimeStage {
    ShowGridPoints,
    SkimGridPoints,
    ShowGridMeshes,
    InterpolateMesh,
    NormalizeMesh,
}

pub struct Timings {
    pub timings: [f32; 5],
    pub delays: [f32; 5],
}

impl Timings {
    pub fn get_time_in_stage(&self, stage: TimeStage, time: f32) -> f32 {
        let stage_idx = stage as usize;
        if stage_idx > self.timings.len() || stage_idx > self.delays.len() {
            panic!("Timing Stage was either to high or timings and delays isn't filled correctly");
        }

        let mut offset = 0.0;

        for i in 0..(stage_idx + 1) {
            offset += self.delays[i];
            if i != stage_idx {
                offset += self.timings[i];
            }
        }

        return ((time - offset) / self.timings[stage_idx]).clamp(0.0, 1.0);
    }
}

pub fn smoothstep(t: f32) -> f32 {
    if t < 0.0 { return 0.0; }
    if t > 1.0 { return 1.0; }

    return t * t * (3.0 - 2.0 * t);
}