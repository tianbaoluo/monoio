mod latency_stat;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use diatomic_waker::WakeSink;
use minstant::{Atomic, Instant};
use crate::latency_stat::{LatencyData, LatencyStat};

const ROUND: usize = 10000;

fn main() {
  let time = Arc::new(Atomic::new(Instant::ZERO));
  let mut wake_sink = WakeSink::new();

  let wake_src = wake_sink.source();
  let send_time = time.clone();
  let sender = std::thread::spawn(move || {
    for _ in 0..ROUND {
      std::thread::sleep(Duration::from_millis(100));
      send_time.store(Instant::now(), Ordering::Relaxed);
      wake_src.notify();
    }
  });

  std::thread::spawn(move || {
    let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
      .with_entries(256)
      .enable_timer()
      .build()
      .unwrap();
    rt.block_on(async move {
      let mut latency_stat = LatencyStat::with_max(10_000);
      for _ in 0..ROUND {
        let latency_us = wake_sink.wait_until(|| {
          let t = time.swap(Instant::ZERO, Ordering::Relaxed);
          if t > Instant::ZERO {
            Some(t.elapsed().as_micros())
          } else {
            None
          }
        }).await;
        latency_stat.record_latency(latency_us as u64);
        // total_latency_us += latency_us;
        // num += 1;
        // println!("latency-us: {}\tavg: {}", latency_us, total_latency_us / num);
      }

      let mut perf_data = LatencyData::new();
      latency_stat.evaluate(&mut perf_data);
      println!("latency: {}", perf_data);
    })
  }).join().unwrap();

  sender.join().unwrap();
}