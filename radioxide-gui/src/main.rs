mod i18n;
mod knob;
mod styles;
mod theme;
mod widgets;

use iced::widget::{Space, button, column, container, row, slider, svg, text, text_input};
use iced::{Element, Font, Length, Task};
use radioxide_proto::{
    Agc, Band, DEFAULT_ADDR, Mode, RadioCommand, RadioStatus, RadioxideMessage, RadioxideResponse,
    Vfo,
};
use radioxide_transports::tcp;

use i18n::I18n;
use styles::*;
use theme::*;

const MONO_FONT: Font = Font {
    family: iced::font::Family::Name("JetBrains Mono"),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

fn main() -> iced::Result {
    iced::application("Radioxide", RadioxideGUI::update, RadioxideGUI::view)
        .font(include_bytes!("../resources/fonts/JetBrainsMono-Bold.ttf").as_slice())
        .theme(|_| radioxide_theme())
        .window_size(iced::Size::new(840.0, 540.0))
        .run_with(|| {
            let gui = RadioxideGUI::new();
            let cmd = Task::perform(
                async { send_cmd(RadioCommand::GetStatus).await },
                Message::Response,
            );
            (gui, cmd)
        })
}

struct RadioxideGUI {
    status: RadioStatus,
    freq_input: String,
    connected: bool,
    last_message: String,
    i18n: I18n,
    knob_on_left: bool,
}

#[derive(Debug, Clone)]
enum Message {
    BandSelected(Band),
    ModeSelected(Mode),
    AgcSelected(Agc),
    VfoSelected(Vfo),
    FreqInputChanged(String),
    FreqSubmit,
    PowerChanged(u8),
    VolumeChanged(u8),
    TunePressed,
    PttToggle,
    RefreshStatus,
    KnobTurned(knob::KnobMessage),
    ToggleKnobSide,
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
            last_message: String::new(),
            i18n: I18n::new(),
            knob_on_left: false,
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
            Message::VfoSelected(vfo) => Self::send(RadioCommand::SetVfo(vfo)),
            Message::FreqInputChanged(val) => {
                self.freq_input = val;
                Task::none()
            }
            Message::FreqSubmit => {
                if let Ok(hz) = self.freq_input.parse::<u64>() {
                    Self::send(RadioCommand::SetFrequency(hz))
                } else {
                    self.last_message = self.i18n.t("invalid-freq");
                    Task::none()
                }
            }
            Message::PowerChanged(pct) => Self::send(RadioCommand::SetPower(pct)),
            Message::VolumeChanged(pct) => Self::send(RadioCommand::SetVolume(pct)),
            Message::TunePressed => Self::send(RadioCommand::Tune),
            Message::PttToggle => {
                if self.status.ptt {
                    Self::send(RadioCommand::PttOff)
                } else {
                    Self::send(RadioCommand::PttOn)
                }
            }
            Message::RefreshStatus => Self::send(RadioCommand::GetStatus),
            Message::KnobTurned(knob::KnobMessage::FreqDelta(delta)) => {
                let new_freq =
                    (self.status.frequency_hz as i64 + delta).clamp(30_000, 60_000_000) as u64;
                self.status.frequency_hz = new_freq;
                self.freq_input = new_freq.to_string();
                Self::send(RadioCommand::SetFrequency(new_freq))
            }
            Message::ToggleKnobSide => {
                self.knob_on_left = !self.knob_on_left;
                Task::none()
            }
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
        let knob_widget = self.view_knob_panel();
        let freq_panel = self.view_frequency_panel();
        let freq_row: Element<'_, Message> = if self.knob_on_left {
            row![knob_widget, freq_panel]
                .spacing(8)
                .width(Length::Fill)
                .into()
        } else {
            row![freq_panel, knob_widget]
                .spacing(8)
                .width(Length::Fill)
                .into()
        };

        let content = column![
            self.view_header(),
            freq_row,
            row![self.view_band_panel(), self.view_mode_panel(),]
                .spacing(8)
                .width(Length::Fill),
            row![
                self.view_vfo_panel(),
                self.view_agc_panel(),
                self.view_controls_panel(),
            ]
            .spacing(8)
            .width(Length::Fill),
            self.view_sliders_panel(),
            self.view_status_bar(),
        ]
        .spacing(8)
        .padding(12)
        .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let led_color = if self.connected { LED_GREEN } else { LED_RED };
        let led = text("●").size(14).color(led_color);

        let logo_handle =
            svg::Handle::from_memory(include_bytes!("../../assets/logo.svg").as_slice());
        let logo = svg(logo_handle).height(Length::Fixed(44.0));

        let refresh_btn = button(text(self.i18n.t("refresh")).size(12))
            .on_press(Message::RefreshStatus)
            .style(action_button)
            .padding([4, 10]);

        row![led, logo, Space::with_width(Length::Fill), refresh_btn,]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    fn view_frequency_panel(&self) -> Element<'_, Message> {
        let freq_text = text(format_frequency(self.status.frequency_hz))
            .font(MONO_FONT)
            .size(48)
            .color(FREQ_GREEN);

        let hz_label = text(self.i18n.t("freq-hz"))
            .font(MONO_FONT)
            .size(20)
            .color(FREQ_GREEN);

        let band_mode = text(format!(
            "VFO-{}  {}  {}",
            self.status.vfo, self.status.band, self.status.mode
        ))
        .size(16)
        .color(TEXT_DIM);

        let freq_display = row![
            freq_text,
            hz_label,
            Space::with_width(Length::Fill),
            band_mode,
        ]
        .spacing(8)
        .align_y(iced::Alignment::End);

        let freq_input = text_input(&self.i18n.t("freq-placeholder"), &self.freq_input)
            .on_input(Message::FreqInputChanged)
            .on_submit(Message::FreqSubmit)
            .style(freq_input_style)
            .font(MONO_FONT)
            .size(14)
            .width(Length::Fixed(180.0));

        let set_btn = button(text(self.i18n.t("freq-set")).size(13))
            .on_press(Message::FreqSubmit)
            .style(action_button)
            .padding([4, 12]);

        let input_row = row![
            text(self.i18n.t("freq-label")).size(13).color(TEXT_DIM),
            freq_input,
            set_btn,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        container(column![freq_display, input_row,].spacing(10).padding(16))
            .width(Length::Fill)
            .style(freq_display_panel)
            .into()
    }

    fn view_knob_panel(&self) -> Element<'_, Message> {
        let knob_canvas: Element<'_, knob::KnobMessage> = knob::view_knob();
        let knob_mapped: Element<'_, Message> = knob_canvas.map(Message::KnobTurned);

        let side_label = if self.knob_on_left { "L" } else { "R" };
        let swap_btn = button(text(side_label).size(10))
            .on_press(Message::ToggleKnobSide)
            .style(action_button)
            .padding([2, 6]);

        container(
            column![knob_mapped, swap_btn]
                .spacing(4)
                .align_x(iced::Alignment::Center),
        )
        .style(panel_style)
        .padding(8)
        .into()
    }

    fn view_band_panel(&self) -> Element<'_, Message> {
        let label = text(self.i18n.t("band-label")).size(11).color(TEXT_DIM);

        let buttons =
            widgets::toggle_button_row(&ALL_BANDS, self.status.band, Message::BandSelected, 5);

        container(column![label, buttons].spacing(6).padding(10))
            .width(Length::Fill)
            .style(panel_style)
            .into()
    }

    fn view_mode_panel(&self) -> Element<'_, Message> {
        let label = text(self.i18n.t("mode-label")).size(11).color(TEXT_DIM);

        let buttons =
            widgets::toggle_button_row(&ALL_MODES, self.status.mode, Message::ModeSelected, 4);

        container(column![label, buttons].spacing(6).padding(10))
            .width(Length::Fill)
            .style(panel_style)
            .into()
    }

    fn view_agc_panel(&self) -> Element<'_, Message> {
        let label = text(self.i18n.t("agc-label")).size(11).color(TEXT_DIM);

        let buttons =
            widgets::toggle_button_row(&ALL_AGC, self.status.agc, Message::AgcSelected, 4);

        container(column![label, buttons].spacing(6).padding(10))
            .width(Length::Fill)
            .style(panel_style)
            .into()
    }

    fn view_vfo_panel(&self) -> Element<'_, Message> {
        let label = text(self.i18n.t("vfo-label")).size(11).color(TEXT_DIM);

        let buttons =
            widgets::toggle_button_row(&ALL_VFOS, self.status.vfo, Message::VfoSelected, 2);

        container(column![label, buttons].spacing(6).padding(10))
            .width(Length::Fill)
            .style(panel_style)
            .into()
    }

    fn view_controls_panel(&self) -> Element<'_, Message> {
        let label = text(self.i18n.t("controls-label")).size(11).color(TEXT_DIM);

        let tune_btn = button(text(self.i18n.t("tune")).size(14))
            .on_press(Message::TunePressed)
            .style(tune_button(self.status.tuning))
            .padding([6, 20]);

        let ptt_label = if self.status.ptt {
            self.i18n.t("ptt-tx")
        } else {
            self.i18n.t("ptt")
        };
        let ptt_btn = button(text(ptt_label).size(14))
            .on_press(Message::PttToggle)
            .style(ptt_button(self.status.ptt))
            .padding([6, 20]);

        container(
            column![label, row![tune_btn, ptt_btn,].spacing(8),]
                .spacing(6)
                .padding(10),
        )
        .width(Length::Fill)
        .style(panel_style)
        .into()
    }

    fn view_sliders_panel(&self) -> Element<'_, Message> {
        let power_label = text(format!(
            "{} {}%",
            self.i18n.t("power-label"),
            self.status.power
        ))
        .size(13)
        .color(TEXT_DIM);

        let power_slider =
            slider(0..=100, self.status.power, Message::PowerChanged).style(radio_slider);

        let vol_label = text(format!(
            "{} {}%",
            self.i18n.t("volume-label"),
            self.status.volume
        ))
        .size(13)
        .color(TEXT_DIM);

        let vol_slider =
            slider(0..=100, self.status.volume, Message::VolumeChanged).style(radio_slider);

        container(
            row![
                column![power_label, power_slider,]
                    .spacing(4)
                    .width(Length::Fill),
                column![vol_label, vol_slider,]
                    .spacing(4)
                    .width(Length::Fill),
            ]
            .spacing(20)
            .padding(10),
        )
        .width(Length::Fill)
        .style(panel_style)
        .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let conn_text = if self.connected {
            self.i18n.t("connected")
        } else {
            self.i18n.t("disconnected")
        };
        let conn_color = if self.connected { LED_GREEN } else { LED_RED };

        container(
            row![
                text(conn_text).size(12).color(conn_color),
                text(" │ ").size(12).color(TEXT_DIM),
                text(&self.last_message).size(12).color(TEXT_DIM),
            ]
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .style(status_bar_style)
        .padding([6, 4])
        .into()
    }
}

/// Format frequency for display (e.g., "14,074,000").
fn format_frequency(hz: u64) -> String {
    let s = hz.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

async fn send_cmd(cmd: RadioCommand) -> Result<RadioxideResponse, String> {
    let msg = RadioxideMessage { command: cmd };
    // iced uses its own async executor (not Tokio), so we need a Tokio runtime
    // for the TCP networking operations.
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?
        .block_on(tcp::send_message(DEFAULT_ADDR, &msg))
        .map_err(|e| e.to_string())
}

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

const ALL_VFOS: [Vfo; 2] = [Vfo::A, Vfo::B];
