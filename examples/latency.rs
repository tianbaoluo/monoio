mod latency_stat;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};
use std::time::Duration;
use diatomic_waker::WakeSink;
use minstant::{Atomic, Instant};
use monoio::Runtime;
use monoio::time::TimeDriver;
use crate::latency_stat::{LatencyData, LatencyStat};

const ROUND: usize = 10000;
const RUN_CPU: u32 = 3;

fn main() {
  let time = Arc::new(Atomic::new(Instant::ZERO));
  let mut wake_sink = WakeSink::new();

  let wake_src = wake_sink.source();
  let send_time = time.clone();
  let sender = std::thread::spawn(move || {
    for _ in 0..ROUND {
      std::thread::sleep(Duration::from_millis(5));
      send_time.store(Instant::now(), Ordering::Relaxed);
      wake_src.notify();
    }
  });

  std::thread::spawn(move || {
    rt().block_on(async move {
      // monoio::spawn_sub(Pending);
      // monoio::spawn_sub(Pending);
      // println!("1. {:?}", monoio::task_metric());
      // monoio::spawn_sub(async move {
      //   loop {
      //     monoio::time::sleep(Duration::from_millis(100)).await;
      //     // println!("{:?}", monoio::task_metric());
      //   }
      // });
      // println!("2. {:?}", monoio::task_metric());
      let mut latency_stat = LatencyStat::with_max(10_000);
      for i in 0..ROUND {
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
        // if i % 100 == 0 {
        //   println!("loop {:?}", monoio::task_metric());
        // }
      }

      // println!("3. {:?}", monoio::task_metric());
      let mut perf_data = LatencyData::new();
      latency_stat.evaluate(&mut perf_data);
      println!("latency: {}", perf_data);
    })
  }).join().unwrap();

  sender.join().unwrap();
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

struct Pending;

impl Future for Pending {
  type Output = ();

  fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Pending
  }
}
