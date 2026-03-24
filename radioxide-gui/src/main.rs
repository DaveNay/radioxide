use iced::widget::{button, column, container, pick_list, row, slider, text, text_input};
use iced::{Element, Length, Task};
use radioxide_proto::{
    Agc, Band, Mode, RadioCommand, RadioStatus, RadioxideMessage, RadioxideResponse, DEFAULT_ADDR,
};
use radioxide_transports::tcp;

fn main() -> iced::Result {
    iced::application("Radioxide", RadioxideGUI::update, RadioxideGUI::view)
        .run_with(|| {
            let gui = RadioxideGUI::new();
            let cmd = Task::perform(async { send_cmd(RadioCommand::GetStatus).await }, |r| {
                Message::Response(r)
            });
            (gui, cmd)
        })
}

struct RadioxideGUI {
    status: RadioStatus,
    freq_input: String,
    connected: bool,
    last_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    // Band/mode selectors
    BandSelected(Band),
    ModeSelected(Mode),
    AgcSelected(Agc),
    // Frequency entry
    FreqInputChanged(String),
    FreqSubmit,
    // Sliders
    PowerChanged(u8),
    VolumeChanged(u8),
    // Buttons
    TunePressed,
    PttOn,
    PttOff,
    RefreshStatus,
    // Response from daemon
    Response(Result<RadioxideResponse, String>),
}

impl RadioxideGUI {
    fn new() -> Self {
        let status = RadioStatus::default();
        let freq = status.frequency_hz.to_string();
        Self {
            status,
            freq_input: freq,
            connected: false,
            last_message: "Connecting...".into(),
        }
    }

    fn send(cmd: RadioCommand) -> Task<Message> {
        Task::perform(async move { send_cmd(cmd).await }, Message::Response)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BandSelected(band) => Self::send(RadioCommand::SetBand(band)),
            Message::ModeSelected(mode) => Self::send(RadioCommand::SetMode(mode)),
            Message::AgcSelected(agc) => Self::send(RadioCommand::SetAgc(agc)),
            Message::FreqInputChanged(val) => {
                self.freq_input = val;
                Task::none()
            }
            Message::FreqSubmit => {
                if let Ok(hz) = self.freq_input.parse::<u64>() {
                    Self::send(RadioCommand::SetFrequency(hz))
                } else {
                    self.last_message = "Invalid frequency".into();
                    Task::none()
                }
            }
            Message::PowerChanged(pct) => Self::send(RadioCommand::SetPower(pct)),
            Message::VolumeChanged(pct) => Self::send(RadioCommand::SetVolume(pct)),
            Message::TunePressed => Self::send(RadioCommand::Tune),
            Message::PttOn => Self::send(RadioCommand::PttOn),
            Message::PttOff => Self::send(RadioCommand::PttOff),
            Message::RefreshStatus => Self::send(RadioCommand::GetStatus),
            Message::Response(Ok(resp)) => {
                self.connected = true;
                self.last_message = resp.message.clone();
                if let Some(st) = resp.status {
                    self.freq_input = st.frequency_hz.to_string();
                    self.status = st;
                }
                Task::none()
            }
            Message::Response(Err(e)) => {
                self.connected = false;
                self.last_message = format!("Error: {e}");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let conn_status = if self.connected { "Connected" } else { "Disconnected" };

        // Frequency section
        let freq_display = text(format_frequency(self.status.frequency_hz)).size(32);
        let freq_row = row![
            text("Freq (Hz): ").size(16),
            text_input("14074000", &self.freq_input)
                .on_input(Message::FreqInputChanged)
                .on_submit(Message::FreqSubmit)
                .width(Length::Fixed(150.0)),
            button("Set").on_press(Message::FreqSubmit),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Band selector
        let band_row = row![
            text("Band: ").size(16),
            pick_list(&ALL_BANDS[..], Some(self.status.band), Message::BandSelected),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Mode selector
        let mode_row = row![
            text("Mode: ").size(16),
            pick_list(&ALL_MODES[..], Some(self.status.mode), Message::ModeSelected),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // AGC selector
        let agc_row = row![
            text("AGC:  ").size(16),
            pick_list(&ALL_AGC[..], Some(self.status.agc), Message::AgcSelected),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Power slider
        let power_row = row![
            text(format!("Power: {}%", self.status.power)).size(16),
            slider(0..=100, self.status.power, Message::PowerChanged),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Volume slider
        let volume_row = row![
            text(format!("Volume: {}%", self.status.volume)).size(16),
            slider(0..=100, self.status.volume, Message::VolumeChanged),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Control buttons
        let controls = row![
            button("Tune").on_press(Message::TunePressed),
            button("PTT ON").on_press(Message::PttOn),
            button("PTT OFF").on_press(Message::PttOff),
            button("Refresh").on_press(Message::RefreshStatus),
        ]
        .spacing(10);

        // Status bar
        let status_bar = row![
            text(conn_status).size(14),
            text(" | ").size(14),
            text(&self.last_message).size(14),
        ];

        let content = column![
            freq_display,
            freq_row,
            band_row,
            mode_row,
            agc_row,
            power_row,
            volume_row,
            controls,
            status_bar,
        ]
        .spacing(12)
        .padding(20)
        .width(Length::Fill);

        container(content).into()
    }
}

/// Format frequency for display (e.g., 14,074,000 Hz).
fn format_frequency(hz: u64) -> String {
    let s = hz.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    format!("{} Hz", result.chars().rev().collect::<String>())
}

async fn send_cmd(cmd: RadioCommand) -> Result<RadioxideResponse, String> {
    let msg = RadioxideMessage { command: cmd };
    tcp::send_message(DEFAULT_ADDR, &msg)
        .await
        .map_err(|e| e.to_string())
}

// Pick-list data — iced needs &[T] where T: Display + Eq
const ALL_BANDS: [Band; 13] = [
    Band::Band160m,
    Band::Band80m,
    Band::Band60m,
    Band::Band40m,
    Band::Band30m,
    Band::Band20m,
    Band::Band17m,
    Band::Band15m,
    Band::Band12m,
    Band::Band10m,
    Band::Band6m,
    Band::Band2m,
    Band::Band70cm,
];

const ALL_MODES: [Mode; 8] = [
    Mode::LSB,
    Mode::USB,
    Mode::CW,
    Mode::AM,
    Mode::FM,
    Mode::Digital,
    Mode::CWR,
    Mode::DigitalR,
];

const ALL_AGC: [Agc; 4] = [Agc::Off, Agc::Fast, Agc::Medium, Agc::Slow];
