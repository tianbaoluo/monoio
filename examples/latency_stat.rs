use std::fmt;

pub struct LatencyData {
  percentiles: Vec<f64>, // [25,50,75,90,95,99]
  count: usize,
  min: u64,
  max: u64,
  avg: f64,
  latencies: Vec<u64>,
}

impl fmt::Display for LatencyData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.count > 0 {
      for (p, l) in self.percentiles.iter().zip(self.latencies.iter()) {
        write!(f, "{}%: {}\t", *p, *l)?;
      }
      write!(f, "#total: {}\tmin: {}\tmax: {}\tavg: {:.1}", self.count, self.min, self.max, self.avg)?;
    } else {
      for p in self.percentiles.iter() {
        write!(f, "{}%: -\t", *p)?;
      }
      write!(f, "#total: 0\tmin: -\tmax: -\tavg: -")?;
    }
    Ok(())
  }
}

impl LatencyData {
  pub fn new() -> Self {
    let percentiles = vec![25.0, 50.0, 75.0, 90.0, 95.0, 99.0];
    Self::with_percentiles(percentiles)
  }

  pub fn with_percentiles(percentiles: Vec<f64>) -> Self {
    let latencies = vec![0; percentiles.len()];
    LatencyData {
      percentiles,
      count: 0,
      min: 0,
      max: 0,
      avg: 0.0,
      latencies,
    }
  }
}

pub struct LatencyStat {
  min: usize,
  max: usize,
  count: usize,
  sum: f64,
  counter: Vec<usize>,
}

impl LatencyStat {
  pub fn with_max(max: u64) -> Self {
    LatencyStat {
      min: max as usize,
      max: 0,
      count: 0,
      sum: 0.0,
      counter: vec![0; (max+1) as usize],
    }
  }

  #[inline(always)]
  pub fn clear(&mut self) {
    self.min = self.counter.len() - 1;
    self.max = 0;
    self.count = 0;
    self.sum = 0.0;
    self.counter.fill(0);
  }

  pub fn record_latency(&mut self, latency: u64) {
    let latency = latency as usize;
    self.min = self.min.min(latency);
    self.max = self.max.max(latency);
    self.count += 1;
    self.sum += latency as f64;
    if let Some(c) = self.counter.get_mut(latency) {
      *c += 1;
    }
  }

  pub fn evaluate(& self, data: &mut LatencyData) {
    data.count = self.count;

    #[cold]
    if self.count == 0 {
      return;
    }

    data.min = self.min as u64;
    data.max = self.max as u64;
    data.avg = self.sum / (self.count as f64);

    data.latencies.fill(0);
    unsafe {
      let mut i = self.min;
      let mut sum_i = *self.counter.get_unchecked(i);
      for (pi, p) in data.percentiles.iter().enumerate() {
        let sum = (*p * self.count as f64 / 100.0).ceil() as usize;
        while sum_i < sum && i < self.max {
          i += 1;
          sum_i += *self.counter.get_unchecked(i);
        }
        *data.latencies.get_unchecked_mut(pi) = i as u64;
      }
    }
  }

  #[inline(always)]
  pub fn count(&self) -> usize {
    self.count
  }

  #[inline(always)]
  pub fn min(&self) -> u64 {
    self.min as u64
  }

  #[inline(always)]
  pub fn max(&self) -> u64 {
    self.max as u64
  }

  #[inline(always)]
  pub fn avg(&self) -> f64 {
    self.sum / (self.count as f64).max(1.0)
  }

  pub fn evaluation(& self, percentiles: &[f64]) -> Vec<u64> {
    let mut latency = vec![0; percentiles.len()];
    if self.count == 0 {
      return latency;
    }

    unsafe {
      let mut i = self.min;
      let mut sum_i = *self.counter.get_unchecked(i);
      for (pi, p) in percentiles.iter().enumerate() {
        let sum = (*p * self.count as f64 / 100.0).ceil() as usize;
        while sum_i < sum && i < self.max {
          i += 1;
          sum_i += *self.counter.get_unchecked(i);
        }
        *latency.get_unchecked_mut(pi) = i as u64;
        // latency.push(i as u64);
      }
    }
    latency
  }
}