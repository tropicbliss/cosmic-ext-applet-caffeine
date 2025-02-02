use cosmic::app::Core;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Command, Length, Limits};
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::iced_widget::Column;
use cosmic::widget::container;
use cosmic::{Element, Theme};
use cosmic_time::once_cell::sync::Lazy;
use cosmic_time::{anim, chain, id, Instant, Timeline};

use crate::caffeine::Caffeine;
use crate::fl;

static SHOW_MEDIA_CONTROLS: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);

const ID: &str = "net.tropicbliss.CosmicExtAppletCaffeine";
const ICON: &str = "net.tropicbliss.CaffeineIcon";

#[derive(Default)]
pub struct Window {
    core: Core,
    popup: Option<Id>,
    caffeine: Caffeine,
    is_stay_awake: bool,
    timeline: Timeline,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Caffeine(chain::Toggler, bool),
    SetStayAwake(bool),
    Frame(Instant),
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
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Caffeine(chain, is_stay_awake) => {
                self.timeline.set_chain(chain).start();
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
                    self.caffeine.decaffeinate()
                } {
                    tracing::error!("Failed to stay awake: {e:?}");
                }
                return cosmic::command::message(Message::SetStayAwake(
                    self.caffeine.is_caffeinated(),
                ));
            }
            Message::SetStayAwake(is_stay_awake) => {
                self.is_stay_awake = is_stay_awake;
            }
            Message::Frame(now) => self.timeline.now(now),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button(ICON)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let mut content = Column::new();
        content = content.push(
            container(
                anim!(
                    SHOW_MEDIA_CONTROLS,
                    &self.timeline,
                    Some(fl!("stay-awake").to_string()),
                    self.is_stay_awake,
                    Message::Caffeine,
                )
                .text_size(14)
                .width(Length::Fill),
            )
            .padding([8, 24]),
        );
        self.core
            .applet
            .popup_container(content.padding([9, 0]))
            .into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        self.timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now))
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
