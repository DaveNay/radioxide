//! End-to-end integration tests: start a TCP server with DummyRadio,
//! send commands via tcp::send_message, and verify responses.

use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use radioxide_proto::*;
use radioxide_transports::tcp;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(18600);

fn next_addr() -> String {
    let port = PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("127.0.0.1:{port}")
}

/// Start a TCP server backed by DummyRadio, return the address.
async fn start_test_server() -> String {
    let addr = next_addr();
    let server_addr = addr.clone();

    tokio::spawn(async move {
        // We can't import DummyRadio directly from the daemon binary crate,
        // so we use a simple in-memory handler that mimics basic behavior.
        let status = Arc::new(tokio::sync::Mutex::new(RadioStatus::default()));

        let handler = move |msg: RadioxideMessage| {
            let status = status.clone();
            async move {
                let mut st = status.lock().await;
                match msg.command {
                    RadioCommand::GetStatus => RadioxideResponse {
                        success: true,
                        message: "Radio status".into(),
                        status: Some(st.clone()),
                    },
                    RadioCommand::SetFrequency(hz) => {
                        st.frequency_hz = hz;
                        RadioxideResponse {
                            success: true,
                            message: "Frequency set".into(),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::GetFrequency => RadioxideResponse {
                        success: true,
                        message: format!("Frequency: {} Hz", st.frequency_hz),
                        status: Some(st.clone()),
                    },
                    RadioCommand::SetBand(band) => {
                        st.band = band;
                        st.frequency_hz = match band {
                            Band::Band40m => 7_074_000,
                            Band::Band20m => 14_074_000,
                            _ => st.frequency_hz,
                        };
                        RadioxideResponse {
                            success: true,
                            message: format!("Band set to {band}"),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::SetMode(mode) => {
                        st.mode = mode;
                        RadioxideResponse {
                            success: true,
                            message: format!("Mode set to {mode}"),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::SetPower(pct) => {
                        st.power = pct.min(100);
                        RadioxideResponse {
                            success: true,
                            message: format!("Power set to {pct}%"),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::SetVolume(pct) => {
                        st.volume = pct.min(100);
                        RadioxideResponse {
                            success: true,
                            message: format!("Volume set to {pct}%"),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::SetAgc(agc) => {
                        st.agc = agc;
                        RadioxideResponse {
                            success: true,
                            message: format!("AGC set to {agc}"),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::PttOn => {
                        st.ptt = true;
                        RadioxideResponse {
                            success: true,
                            message: "PTT on".into(),
                            status: Some(st.clone()),
                        }
                    }
                    RadioCommand::PttOff => {
                        st.ptt = false;
                        RadioxideResponse {
                            success: true,
                            message: "PTT off".into(),
                            status: Some(st.clone()),
                        }
                    }
                    _ => RadioxideResponse {
                        success: true,
                        message: format!("OK: {:?}", msg.command),
                        status: Some(st.clone()),
                    },
                }
            }
        };

        let _ = tcp::start_server(&server_addr, handler).await;
    });

    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

fn msg(command: RadioCommand) -> RadioxideMessage {
    RadioxideMessage { command }
}

#[tokio::test]
async fn e2e_get_status() {
    let addr = start_test_server().await;
    let resp = tcp::send_message(&addr, &msg(RadioCommand::GetStatus)).await.unwrap();
    assert!(resp.success);
    let status = resp.status.unwrap();
    assert_eq!(status.frequency_hz, 14_200_000);
    assert_eq!(status.band, Band::Band20m);
    assert_eq!(status.mode, Mode::USB);
}

#[tokio::test]
async fn e2e_set_and_get_frequency() {
    let addr = start_test_server().await;

    let resp = tcp::send_message(&addr, &msg(RadioCommand::SetFrequency(7_074_000)))
        .await
        .unwrap();
    assert!(resp.success);
    assert_eq!(resp.status.unwrap().frequency_hz, 7_074_000);

    let resp = tcp::send_message(&addr, &msg(RadioCommand::GetFrequency))
        .await
        .unwrap();
    assert!(resp.success);
    assert_eq!(resp.status.unwrap().frequency_hz, 7_074_000);
}

#[tokio::test]
async fn e2e_set_band_updates_frequency() {
    let addr = start_test_server().await;

    let resp = tcp::send_message(&addr, &msg(RadioCommand::SetBand(Band::Band40m)))
        .await
        .unwrap();
    assert!(resp.success);
    let status = resp.status.unwrap();
    assert_eq!(status.band, Band::Band40m);
    assert_eq!(status.frequency_hz, 7_074_000);
}

#[tokio::test]
async fn e2e_full_workflow() {
    let addr = start_test_server().await;

    tcp::send_message(&addr, &msg(RadioCommand::SetFrequency(21_074_000))).await.unwrap();
    tcp::send_message(&addr, &msg(RadioCommand::SetMode(Mode::CW))).await.unwrap();
    tcp::send_message(&addr, &msg(RadioCommand::SetPower(75))).await.unwrap();
    tcp::send_message(&addr, &msg(RadioCommand::SetVolume(30))).await.unwrap();
    tcp::send_message(&addr, &msg(RadioCommand::SetAgc(Agc::Fast))).await.unwrap();

    let resp = tcp::send_message(&addr, &msg(RadioCommand::GetStatus)).await.unwrap();
    let status = resp.status.unwrap();
    assert_eq!(status.frequency_hz, 21_074_000);
    assert_eq!(status.mode, Mode::CW);
    assert_eq!(status.power, 75);
    assert_eq!(status.volume, 30);
    assert_eq!(status.agc, Agc::Fast);
}

#[tokio::test]
async fn e2e_ptt_toggle() {
    let addr = start_test_server().await;

    let resp = tcp::send_message(&addr, &msg(RadioCommand::PttOn)).await.unwrap();
    assert!(resp.status.unwrap().ptt);

    let resp = tcp::send_message(&addr, &msg(RadioCommand::PttOff)).await.unwrap();
    assert!(!resp.status.unwrap().ptt);
}
