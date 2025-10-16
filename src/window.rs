use std::{sync::LazyLock, time::Duration};

use cosmic::{
    Element, Task, app,
    applet::{menu_button, padded_control},
    cosmic_theme::Spacing,
    iced::{self, Alignment, Subscription, window},
    theme,
    widget::{self, Column, divider, text},
};
use cosmic_time::{Instant, Timeline, anim, chain, id, once_cell::sync::Lazy};
use iced::{
    platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
    widget::container,
};
use serde::{Deserialize, Serialize};

use crate::caffeine::Caffeine;
use crate::fl;
use crate::timer::Timer;

static STAY_AWAKE_CONTROLS: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);
static REMEMBER_STATE_CONTROLS: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);

const ID: &str = "net.tropicbliss.CosmicExtAppletCaffeine";

#[derive(Default)]
pub struct Window {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
    caffeine: Caffeine,
    timeline: Timeline,
    timer: Timer,
    timer_string: Option<String>,
    secondary_window: Option<SecondaryWindow>,
    persistent_state: Config,
}

enum SecondaryWindow {
    CustomDuration { minutes_text: String },
    Settings,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Config {
    remember_state: bool,
    custom_duration_text_box: String,
    timer_state: Option<TimerDuration>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
enum TimerDuration {
    #[default]
    Infinite,
    CustomSeconds(u64),
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    ToggleCaffeine(bool),
    Frame(Instant),
    Tick,
    SetTimer(u64),
    CustomDuration,
    SecondaryWindowBack,
    EnterCustomDuration(String),
    CustomDurationStart,
    RememberStateToggle(chain::Toggler, bool),
    Settings,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn init(core: cosmic::app::Core, _flags: ()) -> (Self, app::Task<Message>) {
        let config: Config =
            confy::load(ID, None).expect("config file should be in a valid format");
        let mut window = Window {
            core,
            persistent_state: config.clone(),
            ..Default::default()
        };
        if config.remember_state {
            if let Some(timer_duration) = config.timer_state {
                if let TimerDuration::CustomSeconds(secs) = timer_duration {
                    window.timer.start(Duration::from_secs(secs));
                }
                window.set_stay_awake(true);
            }
        }
        (window, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);
                    self.timeline = Timeline::new();
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                self.secondary_window = None;
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleCaffeine(is_stay_awake) => {
                self.persistent_state.timer_state = if is_stay_awake {
                    Some(TimerDuration::Infinite)
                } else {
                    None
                };
                self.update_config();
                self.set_stay_awake(is_stay_awake);
            }
            Message::Frame(now) => self.timeline.now(now),
            Message::Tick => {
                self.timer.tick();
                self.timer_string = self.timer.get_formatted_time();
                if self.timer.timer_just_ended() {
                    self.persistent_state.timer_state = None;
                    self.update_config();
                    self.set_stay_awake(false);
                }
            }
            Message::SetTimer(minutes) => {
                let seconds = minutes * 60;
                self.timer.start(Duration::from_secs(seconds));
                self.persistent_state.timer_state = Some(TimerDuration::CustomSeconds(seconds));
                self.update_config();
                self.set_stay_awake(true);
            }
            Message::CustomDuration => {
                self.secondary_window = Some(SecondaryWindow::CustomDuration {
                    minutes_text: self.persistent_state.custom_duration_text_box.clone(),
                });
            }
            Message::SecondaryWindowBack => {
                self.secondary_window = None;
            }
            Message::EnterCustomDuration(input) => {
                if let Some(SecondaryWindow::CustomDuration { minutes_text }) =
                    &mut self.secondary_window
                {
                    if input.is_empty() || input.parse::<u32>().is_ok() {
                        *minutes_text = input.clone();
                        self.persistent_state.custom_duration_text_box = input;
                        self.update_config();
                    }
                }
            }
            Message::CustomDurationStart => {
                if let Some(SecondaryWindow::CustomDuration { minutes_text }) =
                    &self.secondary_window
                {
                    if minutes_text.is_empty() {
                        return Task::none();
                    }
                    let seconds = minutes_text.parse::<u64>().expect("valid u64 minutes text") * 60;
                    self.timer.start(Duration::from_secs(seconds));
                    self.secondary_window = None;
                    self.persistent_state.timer_state = Some(TimerDuration::CustomSeconds(seconds));
                    self.update_config();
                    self.set_stay_awake(true);
                }
            }
            Message::RememberStateToggle(chain, remember_state) => {
                self.timeline.set_chain(chain).start();
                self.persistent_state.remember_state = remember_state;
                self.update_config();
            }
            Message::Settings => {
                self.secondary_window = Some(SecondaryWindow::Settings);
            }
        }
        Task::none()
    }

    fn view(&'_ self) -> Element<'_, Self::Message> {
        const ICON_EMPTY: &str = "net.tropicbliss.CosmicExtAppletCaffeine-empty";
        const ICON_FULL: &str = "net.tropicbliss.CosmicExtAppletCaffeine-full";

        let icon = if self.caffeine.is_caffeinated() {
            ICON_FULL
        } else {
            ICON_EMPTY
        };
        self.core
            .applet
            .icon_button(icon)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&'_ self, _id: window::Id) -> Element<'_, Message> {
        static PRESET_MINUTES: LazyLock<Vec<(String, u64)>> = LazyLock::new(|| {
            vec![
                (fl!("fifteen-minutes"), 15),
                (fl!("thirty-minutes"), 30),
                (fl!("one-hour"), 60 * 1),
                (fl!("two-hours"), 60 * 2),
                (fl!("four-hours"), 60 * 4),
            ]
        });
        let mut content = Column::new();
        let Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;
        if matches!(
            &self.secondary_window,
            Some(SecondaryWindow::CustomDuration { .. }) | None
        ) {
            let mut stay_awake_text = fl!("stay-awake").to_string();
            if let Some(formatted_time) = &self.timer_string {
                stay_awake_text.push_str(&format!(" ({formatted_time})"))
            }
            content = content.push(padded_control(anim!(
                STAY_AWAKE_CONTROLS,
                &self.timeline,
                stay_awake_text,
                self.caffeine.is_caffeinated(),
                |_chain, enable| { Message::ToggleCaffeine(enable) }
            )));
            content = content
                .push(padded_control(divider::horizontal::default()).padding([space_xxs, space_s]));
        }
        match &self.secondary_window {
            Some(SecondaryWindow::CustomDuration { minutes_text }) => {
                content = content.push(padded_control(
                    widget::text_input("", minutes_text)
                        .label(fl!("minutes"))
                        .on_input(Message::EnterCustomDuration),
                ));
                content = content.push(
                    menu_button(text::body(fl!("start"))).on_press(Message::CustomDurationStart),
                );
            }
            Some(SecondaryWindow::Settings) => {
                content = content.push(padded_control(anim!(
                    REMEMBER_STATE_CONTROLS,
                    &self.timeline,
                    fl!("remember-state"),
                    self.persistent_state.remember_state,
                    Message::RememberStateToggle
                )));
                content = content.push(
                    padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
                );
            }
            None => {
                for (translation, minutes) in PRESET_MINUTES.iter() {
                    content = content.push(
                        menu_button(text::body(translation)).on_press(Message::SetTimer(*minutes)),
                    );
                }
                content = content.push(
                    menu_button(text::body(fl!("custom-duration")))
                        .on_press(Message::CustomDuration),
                );
                content = content.push(
                    padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
                );
                content = content
                    .push(menu_button(text::body(fl!("settings"))).on_press(Message::Settings));
            }
        }
        if matches!(
            &self.secondary_window,
            Some(SecondaryWindow::CustomDuration { .. } | SecondaryWindow::Settings)
        ) {
            content = content
                .push(menu_button(text::body(fl!("back"))).on_press(Message::SecondaryWindowBack));
        }
        content = content.align_x(Alignment::Start).padding([8, 0]);
        self.core.applet.popup_container(container(content)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            self.timeline
                .as_subscription()
                .map(|(_, now)| Message::Frame(now)),
        ];
        if self.timer.is_started() {
            subs.push(cosmic::iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick));
        }
        Subscription::batch(subs)
    }

    fn on_app_exit(&mut self) -> Option<Message> {
        if let Err(e) = self.caffeine.cleanup() {
            tracing::error!("Failed to exit gracefully: {e:?}");
        }
        None
    }
}

impl Window {
    fn update_config(&self) {
        confy::store(ID, None, &self.persistent_state).expect("cannot modify config");
    }

    fn set_stay_awake(&mut self, is_stay_awake: bool) {
        tracing::info!(
            "{} stay awake",
            if is_stay_awake {
                "Enabling"
            } else {
                "Disabling"
            }
        );
        let changed = self.caffeine.is_caffeinated() != is_stay_awake;
        match if is_stay_awake {
            self.caffeine
                .caffeinate()
                .and_then(|_| Ok(chain::Toggler::on(STAY_AWAKE_CONTROLS.clone(), 1.)))
        } else {
            self.timer.cancel();
            self.timer_string = None;
            self.caffeine
                .decaffeinate()
                .and_then(|_| Ok(chain::Toggler::off(STAY_AWAKE_CONTROLS.clone(), 1.)))
        } {
            Ok(chain) if changed => self.timeline.set_chain(chain).start(),
            Err(e) => panic!("{e:?}"),
            _ => {}
        }
    }
}
