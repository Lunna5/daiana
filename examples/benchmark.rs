//! Daiana High-Performance WebSocket Benchmark & Stress-Testing Tool.
//!
//! Measures:
//! - Total concurrent clients & rooms
//! - Throughput: Ingress & Egress packets/sec and MB/sec
//! - Latency: Min, Avg, p50, p90, p95, p99, p99.9, Max latency
//! - Connection establishment rate and packet delivery ratio
//! - Scalability across broadcast, unicast, and multicast routing modes

use bytes::Bytes;
use daiana::packet::{WsInPacket, WsPacket};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use serde_json::Value;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    Broadcast,
    Unicast,
    Multicast,
}

impl RoutingMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "unicast" | "uni" => RoutingMode::Unicast,
            "multicast" | "multi" => RoutingMode::Multicast,
            _ => RoutingMode::Broadcast,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RoutingMode::Broadcast => "Broadcast",
            RoutingMode::Unicast => "Unicast",
            RoutingMode::Multicast => "Multicast",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub host: String,
    pub port: u16,
    pub rooms: usize,
    pub clients_per_room: usize,
    pub rate_per_client: u32, // msgs/sec, 0 for uncapped/burst
    pub duration_secs: u64,
    pub payload_size: usize,
    pub mode: RoutingMode,
    pub spawn_server: bool,
    pub max_clients_server: u16,
    pub rate_limit_server: u32,
    pub json_output: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            rooms: 10,
            clients_per_room: 4,
            rate_per_client: 20,
            duration_secs: 5,
            payload_size: 64,
            mode: RoutingMode::Broadcast,
            spawn_server: false,
            max_clients_server: 1000,
            rate_limit_server: 0,
            json_output: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BenchmarkResult {
    pub name: String,
    pub total_clients: usize,
    pub total_rooms: usize,
    pub target_rate_per_client: u32,
    pub duration_secs: f64,
    pub payload_size: usize,
    pub mode: String,

    pub connect_duration_ms: f64,
    pub clients_connected: usize,
    pub connection_errors: usize,

    pub sent_packets: u64,
    pub sent_bytes: u64,
    pub received_packets: u64,
    pub received_bytes: u64,

    pub send_rate_pps: f64,
    pub recv_rate_pps: f64,
    pub send_bandwidth_mbps: f64,
    pub recv_bandwidth_mbps: f64,

    pub expected_recv_packets: u64,
    pub delivery_ratio_pct: f64,

    pub min_latency_us: f64,
    pub avg_latency_us: f64,
    pub p50_latency_us: f64,
    pub p90_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub p999_latency_us: f64,
    pub max_latency_us: f64,
    pub stddev_latency_us: f64,
}

struct TestMetrics {
    sent_packets: AtomicU64,
    sent_bytes: AtomicU64,
    received_packets: AtomicU64,
    received_bytes: AtomicU64,
    dropped_or_err_packets: AtomicU64,
    latencies_us: Mutex<Vec<u64>>,
}

impl TestMetrics {
    fn new() -> Self {
        Self {
            sent_packets: AtomicU64::new(0),
            sent_bytes: AtomicU64::new(0),
            received_packets: AtomicU64::new(0),
            received_bytes: AtomicU64::new(0),
            dropped_or_err_packets: AtomicU64::new(0),
            latencies_us: Mutex::new(Vec::with_capacity(100_000)),
        }
    }
}

/// Spawns an in-process Daiana server with custom benchmarks settings
pub async fn start_in_process_server(
    host: &str,
    port: u16,
    max_clients_on_room: u16,
    max_packets_per_sec: u32,
    max_packet_size_bytes: usize,
) -> Result<actix_web::dev::ServerHandle, Box<dyn std::error::Error>> {
    use actix_web::middleware::{Compress, NormalizePath, TrailingSlash};
    use actix_web::web::Data;
    use actix_web::{App, HttpServer, web};
    use daiana::channel::ChannelManager;
    use daiana::{AppState, service};

    let channel_manager = ChannelManager {
        channels: std::sync::Mutex::new(std::collections::HashMap::new()),
        max_clients_on_room,
    };

    let app_state = Data::new(AppState {
        channel_manager,
        max_packets_per_sec,
        max_packet_size_bytes,
    });

    let bind_addr = format!("{}:{}", host, port);
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::MergeOnly))
            .app_data(app_state.clone())
            .service(service::room::endpoints(web::scope("/room")))
            .service(service::stat::endpoints(web::scope("/stat")))
            .service(service::health::endpoints(web::scope("")))
    })
    .bind(&bind_addr)?
    .workers(workers)
    .run();

    let handle = server.handle();
    tokio::spawn(server);
    tokio::time::sleep(Duration::from_millis(250)).await;
    Ok(handle)
}

/// Check if server is online
async fn check_server_health(http_base: &str) -> bool {
    let http_client = match HttpClient::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match http_client.get(format!("{}/", http_base)).send().await {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

/// Creates rooms concurrently on the server
async fn create_rooms(
    http_client: &HttpClient,
    http_base: &str,
    count: usize,
) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
    let mut tasks = Vec::with_capacity(count);
    for _ in 0..count {
        let client = http_client.clone();
        let url = format!("{}/room/", http_base);
        tasks.push(tokio::spawn(async move {
            let res = client
                .post(&url)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .json::<Value>()
                .await
                .map_err(|e| e.to_string())?;
            let id_str = res["id"]
                .as_str()
                .ok_or_else(|| "Missing room ID in response".to_string())?;
            Uuid::parse_str(id_str).map_err(|e| e.to_string())
        }));
    }

    let mut room_ids = Vec::with_capacity(count);
    for task in tasks {
        let join_res = task.await?;
        let id = join_res.map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        room_ids.push(id);
    }
    Ok(room_ids)
}

/// Run a single benchmark test
pub async fn run_benchmark(
    name: &str,
    config: &BenchmarkConfig,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let http_base = format!("http://{}:{}", config.host, config.port);
    let ws_base = format!("ws://{}:{}", config.host, config.port);

    let total_clients = config.rooms * config.clients_per_room;
    let metrics = Arc::new(TestMetrics::new());
    let http_client = HttpClient::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let connect_start = Instant::now();

    // 1. Create rooms
    let room_ids = create_rooms(&http_client, &http_base, config.rooms).await?;

    // 2. Connect all clients
    let start_barrier = Arc::new(Barrier::new(total_clients + 1));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let connected_clients_count = Arc::new(AtomicUsize::new(0));
    let connection_errors_count = Arc::new(AtomicUsize::new(0));

    let mut client_tasks = Vec::with_capacity(total_clients);

    for &room_id in room_ids.iter() {
        for _ in 0..config.clients_per_room {
            let ws_url = format!("{}/room/{}", ws_base, room_id);
            let metrics = metrics.clone();
            let start_barrier = start_barrier.clone();
            let stop_flag = stop_flag.clone();
            let connected_count = connected_clients_count.clone();
            let error_count = connection_errors_count.clone();
            let config = config.clone();

            let task = tokio::spawn(async move {
                let ws_res = connect_async(&ws_url).await;
                let (ws_stream, _) = match ws_res {
                    Ok(conn) => {
                        connected_count.fetch_add(1, Ordering::Relaxed);
                        conn
                    }
                    Err(_) => {
                        error_count.fetch_add(1, Ordering::Relaxed);
                        start_barrier.wait().await;
                        return;
                    }
                };

                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // Collect peers in the room
                let peers_lock = Arc::new(Mutex::new(Vec::<Uuid>::new()));

                // Initial handshake read to discover peers
                let handshake_timeout = Duration::from_millis(300);
                let handshake_deadline = Instant::now() + handshake_timeout;
                let mut peer_ids: Vec<Uuid> = Vec::new();
                while Instant::now() < handshake_deadline {
                    match tokio::time::timeout(Duration::from_millis(30), ws_receiver.next()).await
                    {
                        Ok(Some(Ok(Message::Binary(bin)))) => {
                            if let Ok(WsPacket::ClientConnected { client_id }) =
                                WsPacket::from_bytes(bin)
                            {
                                peer_ids.push(client_id);
                            }
                        }
                        _ => break,
                    }
                }
                *peers_lock.lock().await = peer_ids;

                // Synchronize with all clients before starting load
                start_barrier.wait().await;

                let test_epoch = Instant::now();

                // Receiver task
                let rx_metrics = metrics.clone();
                let rx_stop = stop_flag.clone();
                let rx_task = tokio::spawn(async move {
                    let mut local_latencies = Vec::with_capacity(10_000);
                    while !rx_stop.load(Ordering::Relaxed) {
                        match tokio::time::timeout(Duration::from_millis(100), ws_receiver.next())
                            .await
                        {
                            Ok(Some(Ok(Message::Binary(bin)))) => {
                                let bin_len = bin.len();
                                if let Ok(WsPacket::Message {
                                    sender_id: _,
                                    payload,
                                }) = WsPacket::from_bytes(bin)
                                {
                                    rx_metrics.received_packets.fetch_add(1, Ordering::Relaxed);
                                    rx_metrics
                                        .received_bytes
                                        .fetch_add(bin_len as u64, Ordering::Relaxed);

                                    // Extract timestamp
                                    if payload.len() >= 8 {
                                        let mut ts_bytes = [0u8; 8];
                                        ts_bytes.copy_from_slice(&payload[0..8]);
                                        let sent_us = u64::from_be_bytes(ts_bytes);
                                        let now_us = test_epoch.elapsed().as_micros() as u64;
                                        if now_us >= sent_us {
                                            local_latencies.push(now_us - sent_us);
                                        }
                                    }
                                }
                            }
                            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => break,
                            Ok(Some(Ok(_))) => {}
                            Ok(Some(Err(_))) => {
                                rx_metrics
                                    .dropped_or_err_packets
                                    .fetch_add(1, Ordering::Relaxed);
                                break;
                            }
                            Err(_) => {
                                // Timeout tick, continue loop
                            }
                        }
                    }

                    if !local_latencies.is_empty() {
                        let mut global_lat = rx_metrics.latencies_us.lock().await;
                        if global_lat.len() < 100_000 {
                            global_lat.extend(local_latencies);
                        }
                    }
                });

                // Sender task
                let payload_len = config.payload_size.max(8);
                let mut base_payload = vec![0xDAu8; payload_len];

                let interval_us = if config.rate_per_client > 0 {
                    1_000_000 / config.rate_per_client as u64
                } else {
                    0
                };

                let mut interval = if interval_us > 0 {
                    Some(tokio::time::interval(Duration::from_micros(interval_us)))
                } else {
                    None
                };

                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(ref mut ticker) = interval {
                        ticker.tick().await;
                    }

                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let now_us = test_epoch.elapsed().as_micros() as u64;
                    base_payload[0..8].copy_from_slice(&now_us.to_be_bytes());

                    let payload = Bytes::copy_from_slice(&base_payload);

                    let packet = match config.mode {
                        RoutingMode::Broadcast => WsInPacket::Broadcast { payload },
                        RoutingMode::Unicast => {
                            let peers = peers_lock.lock().await;
                            let target_id = peers.first().cloned().unwrap_or_else(Uuid::new_v4);
                            WsInPacket::Unicast { target_id, payload }
                        }
                        RoutingMode::Multicast => {
                            let peers = peers_lock.lock().await;
                            let target_ids = peers.clone();
                            WsInPacket::Multicast {
                                target_ids,
                                payload,
                            }
                        }
                    };

                    let bytes = packet.to_bytes();
                    let bytes_len = bytes.len() as u64;

                    if ws_sender.send(Message::Binary(bytes)).await.is_err() {
                        metrics
                            .dropped_or_err_packets
                            .fetch_add(1, Ordering::Relaxed);
                        break;
                    }

                    metrics.sent_packets.fetch_add(1, Ordering::Relaxed);
                    metrics.sent_bytes.fetch_add(bytes_len, Ordering::Relaxed);

                    if interval_us == 0 {
                        tokio::task::yield_now().await;
                    }
                }

                let _ = rx_task.await;
                let _ = ws_sender.close().await;
            });

            client_tasks.push(task);
        }
    }

    let connect_duration = connect_start.elapsed();

    // Release all clients to begin sending simultaneously
    start_barrier.wait().await;
    let test_start = Instant::now();

    // Let the test run for the designated duration
    tokio::time::sleep(Duration::from_secs(config.duration_secs)).await;
    stop_flag.store(true, Ordering::Relaxed);

    // Wait for all client tasks to complete
    let _ = futures_util::future::join_all(client_tasks).await;
    let actual_duration = test_start.elapsed().as_secs_f64();

    // Calculate latency metrics
    let latencies = metrics.latencies_us.lock().await.clone();
    let (min_lat, avg_lat, p50, p90, p95, p99, p999, max_lat, stddev) =
        calculate_latency_stats(&latencies);

    let sent = metrics.sent_packets.load(Ordering::Relaxed);
    let sent_b = metrics.sent_bytes.load(Ordering::Relaxed);
    let recv = metrics.received_packets.load(Ordering::Relaxed);
    let recv_b = metrics.received_bytes.load(Ordering::Relaxed);

    // Expected received packets calculation:
    // In broadcast: each sent packet is delivered to (clients_per_room - 1) peers in that room.
    let expected_recv = match config.mode {
        RoutingMode::Broadcast => {
            if config.clients_per_room > 1 {
                sent * (config.clients_per_room as u64 - 1)
            } else {
                0
            }
        }
        RoutingMode::Unicast => sent,
        RoutingMode::Multicast => {
            if config.clients_per_room > 1 {
                sent * (config.clients_per_room as u64 - 1)
            } else {
                0
            }
        }
    };

    let delivery_ratio = if expected_recv > 0 {
        (recv as f64 / expected_recv as f64) * 100.0
    } else {
        100.0
    };

    let send_rate = sent as f64 / actual_duration;
    let recv_rate = recv as f64 / actual_duration;
    let send_mbps = (sent_b as f64 / (1024.0 * 1024.0)) / actual_duration;
    let recv_mbps = (recv_b as f64 / (1024.0 * 1024.0)) / actual_duration;

    Ok(BenchmarkResult {
        name: name.to_string(),
        total_clients,
        total_rooms: config.rooms,
        target_rate_per_client: config.rate_per_client,
        duration_secs: actual_duration,
        payload_size: config.payload_size,
        mode: config.mode.as_str().to_string(),
        connect_duration_ms: connect_duration.as_secs_f64() * 1000.0,
        clients_connected: connected_clients_count.load(Ordering::Relaxed),
        connection_errors: connection_errors_count.load(Ordering::Relaxed),
        sent_packets: sent,
        sent_bytes: sent_b,
        received_packets: recv,
        received_bytes: recv_b,
        send_rate_pps: send_rate,
        recv_rate_pps: recv_rate,
        send_bandwidth_mbps: send_mbps,
        recv_bandwidth_mbps: recv_mbps,
        expected_recv_packets: expected_recv,
        delivery_ratio_pct: delivery_ratio,
        min_latency_us: min_lat,
        avg_latency_us: avg_lat,
        p50_latency_us: p50,
        p90_latency_us: p90,
        p95_latency_us: p95,
        p99_latency_us: p99,
        p999_latency_us: p999,
        max_latency_us: max_lat,
        stddev_latency_us: stddev,
    })
}

fn calculate_latency_stats(
    samples: &[u64],
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let mut sorted = samples.to_vec();
    sorted.sort_unstable();

    let count = sorted.len();
    let min = sorted[0] as f64;
    let max = sorted[count - 1] as f64;

    let sum: u64 = sorted.iter().sum();
    let avg = sum as f64 / count as f64;

    let percentile = |pct: f64| -> f64 {
        let idx = ((count as f64 * pct / 100.0).round() as usize).min(count - 1);
        sorted[idx] as f64
    };

    let p50 = percentile(50.0);
    let p90 = percentile(90.0);
    let p95 = percentile(95.0);
    let p99 = percentile(99.0);
    let p999 = percentile(99.9);

    let variance: f64 = sorted
        .iter()
        .map(|&v| {
            let diff = v as f64 - avg;
            diff * diff
        })
        .sum::<f64>()
        / count as f64;
    let stddev = variance.sqrt();

    (min, avg, p50, p90, p95, p99, p999, max, stddev)
}

fn format_latency(us: f64) -> String {
    if us < 1000.0 {
        format!("{:.1} µs", us)
    } else if us < 1_000_000.0 {
        format!("{:.2} ms", us / 1000.0)
    } else {
        format!("{:.2} s", us / 1_000_000.0)
    }
}

pub fn print_result_card(res: &BenchmarkResult) {
    println!("\x1b[1;36m┌────────────────────────────────────────────────────────────────────────────┐\x1b[0m");
    println!(
        "\x1b[1;36m│\x1b[0m \x1b[1;33mBENCHMARK RESULT: {:<57}\x1b[0m\x1b[1;36m│\x1b[0m",
        res.name
    );
    println!("\x1b[1;36m├────────────────────────────────────────────────────────────────────────────┤\x1b[0m");
    println!(
        "│ \x1b[1mTopology:\x1b[0m        {:>4} Clients ({:>3} rooms × {:>2} clients) | Mode: {:<12}│",
        res.total_clients,
        res.total_rooms,
        res.total_clients / res.total_rooms.max(1),
        res.mode
    );
    println!(
        "│ \x1b[1mPayload:\x1b[0m         {:>4} Bytes  | Target Rate/Client: {:>4} pkt/s | Time: {:>4.1}s   │",
        res.payload_size,
        res.target_rate_per_client,
        res.duration_secs
    );
    println!(
        "│ \x1b[1mConnection:\x1b[0m      {:>4} Connected in {:>6.2} ms (Errors: {:>2})                   │",
        res.clients_connected, res.connect_duration_ms, res.connection_errors
    );
    println!("\x1b[1;36m├────────────────────────────────────────────────────────────────────────────┤\x1b[0m");
    println!(
        "│ \x1b[1;32mThroughput (Ingress):\x1b[0m   {:>10.1} pkts/sec  | {:>7.2} MB/sec ({:>8} pkts)│",
        res.send_rate_pps,
        res.send_bandwidth_mbps,
        res.sent_packets
    );
    println!(
        "│ \x1b[1;32mThroughput (Egress):\x1b[0m    {:>10.1} pkts/sec  | {:>7.2} MB/sec ({:>8} pkts)│",
        res.recv_rate_pps,
        res.recv_bandwidth_mbps,
        res.received_packets
    );
    println!(
        "│ \x1b[1mDelivery Ratio:\x1b[0m        {:>9.2}% (Received {} of {} expected)    │",
        res.delivery_ratio_pct, res.received_packets, res.expected_recv_packets
    );
    println!("\x1b[1;36m├────────────────────────────────────────────────────────────────────────────┤\x1b[0m");
    println!("│ \x1b[1;35mEnd-to-End Latency Percentiles:\x1b[0m                                            │");
    println!(
        "│   Min:  {:<10} │ Mean: {:<10} │ p50 (Med): {:<10}             │",
        format_latency(res.min_latency_us),
        format_latency(res.avg_latency_us),
        format_latency(res.p50_latency_us)
    );
    println!(
        "│   p90:  {:<10} │ p95:  {:<10} │ p99:       {:<10}             │",
        format_latency(res.p90_latency_us),
        format_latency(res.p95_latency_us),
        format_latency(res.p99_latency_us)
    );
    println!(
        "│   p99.9:{:<10} │ Max:  {:<10} │ StdDev:    {:<10}             │",
        format_latency(res.p999_latency_us),
        format_latency(res.max_latency_us),
        format_latency(res.stddev_latency_us)
    );
    println!("\x1b[1;36m└────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n");
}

pub fn print_comparison_table(results: &[BenchmarkResult]) {
    println!("\n\x1b[1;36m========================================================================================================================\x1b[0m");
    println!("\x1b[1;36m                                          DAIANA BENCHMARK COMPARATIVE SUMMARY                                          \x1b[0m");
    println!("\x1b[1;36m========================================================================================================================\x1b[0m");
    println!(
        "\x1b[1m{:<22} {:>7} {:>6} {:>8} {:>12} {:>12} {:>10} {:>9} {:>9} {:>8}\x1b[0m",
        "Scenario", "Clients", "Rooms", "Rate/Cli", "Ingress pkt/s", "Egress pkt/s", "Bandwidth", "p50 Lat", "p99 Lat", "Delivery"
    );
    println!("------------------------------------------------------------------------------------------------------------------------");
    for r in results {
        let total_bw = r.send_bandwidth_mbps + r.recv_bandwidth_mbps;
        println!(
            "{:<22} {:>7} {:>6} {:>8} {:>12.0} {:>12.0} {:>8.2} MB/s {:>9} {:>9} {:>7.1}%",
            if r.name.len() > 22 { &r.name[..22] } else { &r.name },
            r.total_clients,
            r.total_rooms,
            if r.target_rate_per_client == 0 { "MAX".to_string() } else { format!("{} p/s", r.target_rate_per_client) },
            r.send_rate_pps,
            r.recv_rate_pps,
            total_bw,
            format_latency(r.p50_latency_us),
            format_latency(r.p99_latency_us),
            r.delivery_ratio_pct
        );
    }
    println!("\x1b[1;36m========================================================================================================================\x1b[0m\n");
}

pub async fn run_test_suite(config: &BenchmarkConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1;32m🚀 Running Comprehensive Multi-Scenario Benchmark Suite for Daiana...\x1b[0m\n");

    let mut results = Vec::new();

    let scenarios = vec![
        ("1. Low Load Baseline", 5, 2, 10, 64, 4, RoutingMode::Broadcast),
        ("2. Room Concurrency (20 Rooms)", 20, 3, 20, 64, 4, RoutingMode::Broadcast),
        ("3. High Client Count (100 Clients)", 25, 4, 30, 64, 4, RoutingMode::Broadcast),
        ("4. High Rate Flow (80 pkt/s)", 10, 4, 80, 128, 4, RoutingMode::Broadcast),
        ("5. Medium Payload (1 KB)", 10, 4, 30, 1024, 4, RoutingMode::Broadcast),
        ("6. Large Payload (8 KB)", 5, 4, 20, 8192, 4, RoutingMode::Broadcast),
        ("7. Unicast Direct Routing", 15, 4, 40, 64, 4, RoutingMode::Unicast),
        ("8. Max Flow / Burst Mode", 10, 4, 0, 64, 4, RoutingMode::Broadcast),
    ];

    for (name, rooms, clients_per_room, rate, payload_size, duration, mode) in scenarios {
        let mut sc_config = config.clone();
        sc_config.rooms = rooms;
        sc_config.clients_per_room = clients_per_room;
        sc_config.rate_per_client = rate;
        sc_config.payload_size = payload_size;
        sc_config.duration_secs = duration;
        sc_config.mode = mode;

        print!("👉 Running \x1b[1;33m{}\x1b[0m ({} clients, rate: {} pkt/s)... ", name, rooms * clients_per_room, rate);
        let res = run_benchmark(name, &sc_config).await?;
        println!("\x1b[32mDONE\x1b[0m (Egress: \x1b[1m{:.0} pkt/s\x1b[0m, p50: \x1b[1m{}\x1b[0m)", res.recv_rate_pps, format_latency(res.p50_latency_us));
        print_result_card(&res);
        results.push(res);

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    print_comparison_table(&results);
    Ok(())
}

pub async fn run_ramp_test(config: &BenchmarkConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1;32m🚀 Running Ramp-Up / Saturation Flow Benchmark for Daiana...\x1b[0m\n");
    println!("Progressively increasing packet rates to identify maximum throughput ceiling:\n");

    let mut results = Vec::new();
    let rates = vec![10, 25, 50, 100, 200, 400, 800, 0]; // 0 = max burst

    for rate in rates {
        let label = if rate == 0 {
            "Ramp: Burst (Uncapped)".to_string()
        } else {
            format!("Ramp: {} pkt/s per client", rate)
        };

        let mut sc_config = config.clone();
        sc_config.rate_per_client = rate;
        sc_config.duration_secs = 4;

        print!("👉 Testing \x1b[1;33m{}\x1b[0m... ", label);
        let res = run_benchmark(&label, &sc_config).await?;
        println!(
            "\x1b[32mDONE\x1b[0m -> Ingress: \x1b[1m{:.0} pkt/s\x1b[0m, Egress: \x1b[1m{:.0} pkt/s\x1b[0m, p99 Latency: \x1b[1m{}\x1b[0m",
            res.send_rate_pps, res.recv_rate_pps, format_latency(res.p99_latency_us)
        );
        results.push(res);
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    print_comparison_table(&results);
    Ok(())
}

fn print_help() {
    println!(r#"
Daiana WebSocket Benchmark & Load Testing Tool

USAGE:
    cargo run --release --example benchmark -- [OPTIONS]

OPTIONS:
    --host <IP>                 Server host address (default: 127.0.0.1)
    --port <PORT>               Server port (default: 8080)
    --rooms <N>                 Number of rooms to create (default: 10)
    --clients-per-room <N>      Number of clients per room (default: 4)
    --rate <N>                  Packets/sec sent per client, 0 for max burst (default: 20)
    --duration <SECS>           Duration of test in seconds (default: 5)
    --payload-size <BYTES>      Payload size per packet (default: 64)
    --mode <MODE>               Routing mode: broadcast, unicast, multicast (default: broadcast)
    --suite                     Run full multi-scenario benchmark suite
    --ramp                      Run ramp-up / saturation throughput test
    --spawn-server              Spawn an in-process Daiana server automatically
    --max-clients-server <N>    If spawning server: max clients per room (default: 1000)
    --rate-limit-server <N>     If spawning server: max pkts/sec rate limit, 0 to disable (default: 0)
    --json                      Output metrics in JSON format
    --help, -h                  Display this help message

EXAMPLES:
    # Run default benchmark (40 clients across 10 rooms):
    cargo run --release --example benchmark

    # Run full comprehensive test suite:
    cargo run --release --example benchmark -- --suite

    # Run throughput saturation ramp test:
    cargo run --release --example benchmark -- --ramp

    # Custom high-concurrency test (200 clients, 50 rooms, 50 pkt/s):
    cargo run --release --example benchmark -- --rooms 50 --clients-per-room 4 --rate 50 --duration 10

    # Auto-spawn in-process server with uncapped rate limits:
    cargo run --release --example benchmark -- --spawn-server --suite
"#);
}

fn parse_cli_args() -> (BenchmarkConfig, bool, bool) {
    let args: Vec<String> = env::args().collect();
    let mut config = BenchmarkConfig::default();
    let mut run_suite = false;
    let mut run_ramp = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" if i + 1 < args.len() => {
                config.host = args[i + 1].clone();
                i += 1;
            }
            "--port" if i + 1 < args.len() => {
                if let Ok(p) = args[i + 1].parse() {
                    config.port = p;
                }
                i += 1;
            }
            "--rooms" if i + 1 < args.len() => {
                if let Ok(r) = args[i + 1].parse() {
                    config.rooms = r;
                }
                i += 1;
            }
            "--clients-per-room" if i + 1 < args.len() => {
                if let Ok(c) = args[i + 1].parse() {
                    config.clients_per_room = c;
                }
                i += 1;
            }
            "--rate" if i + 1 < args.len() => {
                if let Ok(r) = args[i + 1].parse() {
                    config.rate_per_client = r;
                }
                i += 1;
            }
            "--duration" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse() {
                    config.duration_secs = d;
                }
                i += 1;
            }
            "--payload-size" if i + 1 < args.len() => {
                if let Ok(s) = args[i + 1].parse() {
                    config.payload_size = s;
                }
                i += 1;
            }
            "--mode" if i + 1 < args.len() => {
                config.mode = RoutingMode::from_str(&args[i + 1]);
                i += 1;
            }
            "--max-clients-server" if i + 1 < args.len() => {
                if let Ok(m) = args[i + 1].parse() {
                    config.max_clients_server = m;
                }
                i += 1;
            }
            "--rate-limit-server" if i + 1 < args.len() => {
                if let Ok(r) = args[i + 1].parse() {
                    config.rate_limit_server = r;
                }
                i += 1;
            }
            "--spawn-server" => {
                config.spawn_server = true;
            }
            "--suite" => {
                run_suite = true;
            }
            "--ramp" => {
                run_ramp = true;
            }
            "--json" => {
                config.json_output = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    (config, run_suite, run_ramp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config, run_suite, run_ramp) = parse_cli_args();

    println!("\x1b[1;36m========================================================================\x1b[0m");
    println!("\x1b[1;36m             DAIANA - Real-Time WebSocket Server Benchmark              \x1b[0m");
    println!("\x1b[1;36m========================================================================\x1b[0m\n");

    let http_base = format!("http://{}:{}", config.host, config.port);
    let mut _server_handle = None;

    // Check if server is already running, or if we need to auto-spawn it
    let server_online = check_server_health(&http_base).await;
    if !server_online {
        println!("ℹ️  No active Daiana server detected at {}. Auto-starting in-process server...", http_base);
        let handle = start_in_process_server(
            &config.host,
            config.port,
            config.max_clients_server,
            config.rate_limit_server,
            65_536,
        )
        .await?;
        println!("\x1b[32m✓\x1b[0m In-process Daiana server successfully initialized at {}\n", http_base);
        _server_handle = Some(handle);
    } else if config.spawn_server {
        println!("⚠️  --spawn-server specified but server is already running on {}. Using existing server.", http_base);
    } else {
        println!("\x1b[32m✓\x1b[0m Connected to existing Daiana server at {}\n", http_base);
    }

    if run_suite {
        run_test_suite(&config).await?;
    } else if run_ramp {
        run_ramp_test(&config).await?;
    } else {
        println!(
            "Running benchmark: {} rooms × {} clients = {} clients (Rate: {} pkt/s per client, Duration: {}s)...",
            config.rooms, config.clients_per_room, config.rooms * config.clients_per_room, config.rate_per_client, config.duration_secs
        );
        let res = run_benchmark("Custom Benchmark Run", &config).await?;
        print_result_card(&res);

        if config.json_output {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "name": res.name,
                "total_clients": res.total_clients,
                "total_rooms": res.total_rooms,
                "target_rate_per_client": res.target_rate_per_client,
                "duration_secs": res.duration_secs,
                "payload_size": res.payload_size,
                "mode": res.mode,
                "send_rate_pps": res.send_rate_pps,
                "recv_rate_pps": res.recv_rate_pps,
                "send_bandwidth_mbps": res.send_bandwidth_mbps,
                "recv_bandwidth_mbps": res.recv_bandwidth_mbps,
                "delivery_ratio_pct": res.delivery_ratio_pct,
                "min_latency_us": res.min_latency_us,
                "avg_latency_us": res.avg_latency_us,
                "p50_latency_us": res.p50_latency_us,
                "p90_latency_us": res.p90_latency_us,
                "p95_latency_us": res.p95_latency_us,
                "p99_latency_us": res.p99_latency_us,
                "p999_latency_us": res.p999_latency_us,
                "max_latency_us": res.max_latency_us,
            }))?);
        }
    }

    Ok(())
}
