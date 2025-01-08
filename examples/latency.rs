mod latency_stat;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use diatomic_waker::WakeSink;
use minstant::{Atomic, Instant};
use monoio::Runtime;
use monoio::time::TimeDriver;
use crate::latency_stat::{LatencyData, LatencyStat};

const ROUND: usize = 100_000;
const RUN_CPU: u32 = 3;

fn main() {
  let time = Arc::new(Atomic::new(Instant::ZERO));
  let mut wake_sink = WakeSink::new();

  let wake_src = wake_sink.source();
  let send_time = time.clone();
  let done = Arc::new(AtomicBool::new(false));
  let done_clone = done.clone();
  let sender = std::thread::spawn(move || {
    for _ in 0..ROUND {
      std::thread::sleep(Duration::from_millis(5));
      send_time.store(Instant::now(), Ordering::Relaxed);
      wake_src.notify();
    }
    println!("[send] done");
    done_clone.store(true, Ordering::Relaxed);
    wake_src.notify();
  });

  let receiver = std::thread::spawn(move || {
    rt().block_on(async move {
      let mut latency_stat = LatencyStat::with_max(10_000);
      for i in 0..ROUND {
        let latency_us = wake_sink.wait_until(|| {
          let t = time.swap(Instant::ZERO, Ordering::Relaxed);
          if t > Instant::ZERO {
            Some(t.elapsed().as_micros())
          } else {
            if done.load(Ordering::Relaxed) {
              Some(u128::MAX)
            } else {
              None
            }
          }
        }).await;
        if latency_us == u128::MAX {
          println!("[receiver] got {}/{} exit", i, ROUND);
          break;
        }
        latency_stat.record_latency(latency_us as u64);
        // total_latency_us += latency_us;
        // num += 1;
        // println!("latency-us: {}\tavg: {}", latency_us, total_latency_us / num);
      }

      let mut perf_data = LatencyData::new();
      latency_stat.evaluate(&mut perf_data);
      println!("latency: {}", perf_data);
    })
  });

  sender.join().unwrap();
  receiver.join().unwrap();
}

#[cfg(target_os = "linux")]
fn rt() -> Runtime<TimeDriver<monoio::IoUringDriver>> {
  monoio::utils::bind_to_cpu_set(vec![RUN_CPU as usize]).unwrap();
  let mut urb = io_uring::IoUring::builder();
  urb.setup_sqpoll(200).setup_sqpoll_cpu(RUN_CPU);

  monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
    .uring_builder(urb)
    .with_entries(256)
    .enable_all()
    .build()
    .unwrap()
}

#[cfg(not(target_os = "linux"))]
fn rt() -> Runtime<TimeDriver<monoio::LegacyDriver>> {
  monoio::RuntimeBuilder::<monoio::LegacyDriver>::new()
    .with_entries(256)
    .enable_timer()
    .build()
    .unwrap()
}