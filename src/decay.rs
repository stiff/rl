/// An implementation of a time-decaying value
pub trait Decay {
    /// Calculate value at time `t`
    fn evaluate(&self, t: f32) -> f32;
}

// TODO: better error types
fn validate(rate: f32, vi: f32, vf: f32) -> Result<(), String> {
    ((rate >= 0.0 && vi > vf) || (rate < 0.0 && vi < vf))
        .then_some(())
        .ok_or_else(|| String::from("`vi - vf` must have same sign as `rate`"))
}

/// A constant value
pub struct Constant {
    value: f32,
}

impl Constant {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl Decay for Constant {
    fn evaluate(&self, _t: f32) -> f32 {
        self.value
    }
}

/// v(t) = v<sub>f</sub> + (v<sub>i</sub> - v<sub>f</sub>) * e<sup>-rt</sup>
pub struct Exponential {
    rate: f32,
    vi: f32,
    vf: f32,
}

impl Exponential {
    pub fn new(rate: f32, vi: f32, vf: f32) -> Result<Self, String> {
        validate(rate, vi, vf)?;
        Ok(Self { rate, vi, vf })
    }
}

impl Decay for Exponential {
    fn evaluate(&self, t: f32) -> f32 {
        let &Self { rate, vi, vf } = self;
        vf + (vi - vf) * (-rate * t).exp()
    }
}

/// v(t) = v<sub>f</sub> + (v<sub>i</sub> - v<sub>f</sub>) / (1 + rt)
pub struct InverseTime {
    rate: f32,
    vi: f32,
    vf: f32,
}

impl InverseTime {
    pub fn new(rate: f32, vi: f32, vf: f32) -> Result<Self, String> {
        validate(rate, vi, vf)?;
        Ok(Self { rate, vi, vf })
    }
}

impl Decay for InverseTime {
    fn evaluate(&self, t: f32) -> f32 {
        let &Self { rate, vi, vf } = self;
        vf + (vi - vf) / (1.0 + rate * t)
    }
}

/// v(t) = max(v<sub>i</sub> - rt, v<sub>f</sub>)
pub struct Linear {
    rate: f32,
    vi: f32,
    vf: f32,
}

impl Linear {
    pub fn new(rate: f32, vi: f32, vf: f32) -> Result<Self, String> {
        validate(rate, vi, vf)?;
        Ok(Self { rate, vi, vf })
    }
}

impl Decay for Linear {
    fn evaluate(&self, t: f32) -> f32 {
        let &Self { rate, vi, vf } = self;
        (vi - rate * t).max(vf)
    }
}

/// v(t) = max(v<sub>i</sub> * r<sup>floor(t/s)</sup>, v<sub>f</sub>)
pub struct Step {
    rate: f32,
    vi: f32,
    vf: f32,
    step: f32,
}

impl Step {
    pub fn new(rate: f32, vi: f32, vf: f32, step: f32) -> Result<Self, String> {
        validate(rate, vi, vf)?;
        Ok(Self { rate, vi, vf, step })
    }
}

impl Decay for Step {
    fn evaluate(&self, t: f32) -> f32 {
        let &Self { rate, vi, vf, step } = self;
        (vi * rate.powf((t / step).floor())).max(vf)
    }
}
