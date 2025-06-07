use std::sync::LazyLock;
use std::time::Duration;

use cosmic::app::Core;
use cosmic::applet::{self, padded_control};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{widget, Command, Length, Limits, Subscription};
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::iced_widget::{row, Column};
use cosmic::widget::{divider, text_input};
use cosmic::{theme, Apply, Element, Theme};
use cosmic_time::once_cell::sync::Lazy;
use cosmic_time::{anim, chain, id, Instant, Timeline};

use crate::caffeine::Caffeine;
use crate::fl;
use crate::timer::Timer;

static STAY_AWAKE_CONTROLS: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);

const ID: &str = "net.tropicbliss.CosmicExtAppletCaffeine";

#[derive(Default)]
pub struct Window {
    core: Core,
    popup: Option<Id>,
    caffeine: Caffeine,
    is_stay_awake: bool,
    timeline: Timeline,
    timer: Timer,
    timer_string: Option<String>,
    custom_duration_window: Option<CustomDurationWindow>,
}

#[derive(Default)]
pub struct CustomDurationWindow {
    minutes_text: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Caffeine(chain::Toggler, bool),
    Frame(Instant),
    SetStayAwake(bool),
    Tick,
    SetTimer(u64),
    CustomDuration,
    CustomDurationBack,
    EnterCustomDuration(String),
    CustomDurationStart,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let window = Window {
            core,
            ..Default::default()
        };
        (window, Command::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    self.timeline = Timeline::new();
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits =
                        Limits::NONE.max_width(372.0).max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                self.custom_duration_window = None;
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::SetStayAwake(is_stay_awake) => {
                tracing::info!(
                    "{} stay awake",
                    if is_stay_awake {
                        "Enabling"
                    } else {
                        "Disabling"
                    }
                );
                if let Err(e) = if is_stay_awake {
                    self.caffeine.caffeinate()
                } else {
                    self.timer.cancel();
                    self.timer_string = None;
                    self.caffeine.decaffeinate()
                } {
                    tracing::error!("Failed to stay awake: {e:?}");
                }
                self.is_stay_awake = self.caffeine.is_caffeinated();
            }
            Message::Caffeine(chain, is_stay_awake) => {
                self.timeline.set_chain(chain).start();
                return cosmic::command::message(Message::SetStayAwake(is_stay_awake));
            }
            Message::Frame(now) => self.timeline.now(now),
            Message::Tick => {
                self.timer.tick();
                self.timer_string = self.timer.get_formatted_time();
                if self.timer.timer_just_ended() {
                    return cosmic::command::message(Message::SetStayAwake(false));
                }
            }
            Message::SetTimer(minutes) => {
                self.timer.start(Duration::from_secs(minutes * 60));
                return cosmic::command::message(Message::SetStayAwake(true));
            }
            Message::CustomDuration => {
                self.custom_duration_window = Some(CustomDurationWindow::default());
            }
            Message::CustomDurationBack => {
                self.custom_duration_window = None;
            }
            Message::EnterCustomDuration(input) => {
                let custom_duration_state = self
                    .custom_duration_window
                    .as_mut()
                    .expect("in custom duration window");
                if input.is_empty() || input.parse::<u32>().is_ok() {
                    custom_duration_state.minutes_text = input;
                }
            }
            Message::CustomDurationStart => {
                let custom_duration_state = self
                    .custom_duration_window
                    .as_ref()
                    .expect("in custom duration window");
                if custom_duration_state.minutes_text.is_empty() {
                    return Command::none();
                }
                let minutes = custom_duration_state.minutes_text.parse::<u64>().unwrap();
                self.timer.start(Duration::from_secs(minutes * 60));
                self.custom_duration_window = None;
                return cosmic::command::message(Message::SetStayAwake(true));
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button(ID)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        static PRESET_MINUTES: LazyLock<Vec<(String, u64)>> = LazyLock::new(|| {
            vec![
                (fl!("fifteen-minutes"), 1),
                (fl!("thirty-minutes"), 30),
                (fl!("one-hour"), 60 * 1),
                (fl!("two-hours"), 60 * 2),
                (fl!("four-hours"), 60 * 4),
            ]
        });
        let mut content = Column::new();
        let mut stay_awake_text = fl!("stay-awake").to_string();
        if let Some(formatted_time) = &self.timer_string {
            stay_awake_text.push_str(&format!(" ({formatted_time})"))
        }
        content = content.push(padded_control(row![anim!(
            STAY_AWAKE_CONTROLS,
            &self.timeline,
            stay_awake_text,
            self.is_stay_awake,
            Message::Caffeine
        )]));
        let Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;
        content = content
            .push(padded_control(divider::horizontal::default()).padding([space_xxs, space_s]));
        if let Some(custom_duration_state) = &self.custom_duration_window {
            content = content.push(
                text_input(fl!("minutes"), &custom_duration_state.minutes_text)
                    .on_input(Message::EnterCustomDuration)
                    .apply(padded_control)
                    .width(Length::Fill),
            );
            content = content.push(
                applet::menu_button(widget::text(fl!("start").to_string()))
                    .on_press(Message::CustomDurationStart),
            );
            content = content.push(
                applet::menu_button(widget::text(fl!("back").to_string()))
                    .on_press(Message::CustomDurationBack),
            );
        } else {
            for (translation, minutes) in PRESET_MINUTES.iter() {
                content = content.push(
                    applet::menu_button(widget::text(translation.to_string()))
                        .on_press(Message::SetTimer(*minutes)),
                );
            }
            content = content.push(
                applet::menu_button(widget::text(fl!("custom-duration").to_string()))
                    .on_press(Message::CustomDuration),
            );
        }
        self.core
            .applet
            .popup_container(content.padding([9, 0]))
            .into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let mut subs = vec![self
            .timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now))];
        if self.timer.is_started() {
            subs.push(cosmic::iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick));
        }
        Subscription::batch(subs)
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }

    fn on_app_exit(&mut self) -> Option<Self::Message> {
        if let Err(e) = self.caffeine.cleanup() {
            tracing::error!("Failed to exit gracefully: {e:?}");
        }
        None
    }
}
