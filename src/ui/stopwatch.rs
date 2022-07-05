#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Stopped,
    Reset,
    Running,
}
impl Default for State {
    fn default() -> Self {
        Self::Stopped
    }
}

mod imp {
    use chrono::Duration;
    use gtk::{
        gio::ListStore,
        glib::{self, clone, subclass::InitializingObject, timeout_add_local},
        prelude::*,
        subclass::prelude::*,
        template_callbacks, Box, Button, CompositeTemplate, Label,
    };
    use he::{traits::ButtonExt as HeButtonExt, Colors, FillButton};
    use log::debug;
    use std::cell::Cell;
    use stopwatch::Stopwatch;

    use crate::lap::StopwatchLap;

    use super::State;

    #[derive(CompositeTemplate)]
    #[template(resource = "/co/tauos/Nixie/stopwatch.ui")]
    pub struct StopwatchPage {
        #[template_child]
        pub time_container: TemplateChild<Box>,

        #[template_child]
        pub hours_label: TemplateChild<Label>,
        #[template_child]
        pub minutes_label: TemplateChild<Label>,
        #[template_child]
        pub seconds_label: TemplateChild<Label>,
        #[template_child]
        pub miliseconds_label: TemplateChild<Label>,

        #[template_child]
        pub start_btn: TemplateChild<FillButton>,
        #[template_child]
        pub clear_btn: TemplateChild<FillButton>,

        pub timer: Cell<Stopwatch>,
        pub state: Cell<State>,
        pub laps: ListStore,
        pub current_lap: Cell<i32>,
    }

    impl Default for StopwatchPage {
        fn default() -> Self {
            Self {
                time_container: TemplateChild::default(),
                hours_label: TemplateChild::default(),
                minutes_label: TemplateChild::default(),
                seconds_label: TemplateChild::default(),
                miliseconds_label: TemplateChild::default(),
                start_btn: TemplateChild::default(),
                clear_btn: TemplateChild::default(),
                timer: Cell::new(Stopwatch::new()),
                state: Cell::new(State::Stopped),
                laps: ListStore::new(StopwatchLap::type_(&StopwatchLap::default())),
                current_lap: Cell::new(0),
            }
        }
    }

    #[template_callbacks]
    impl StopwatchPage {
        fn start(&self) {
            let mut sw = self.timer.get();
            sw.start();
            self.timer.replace(sw);
            self.state.replace(State::Running);

            self.start_btn.set_label("Pause");
            self.start_btn.set_color(Colors::Yellow);

            self.clear_btn.set_label("Lap");
            self.clear_btn.set_sensitive(true);
            self.clear_btn.set_color(Colors::Purple);

            self.time_container.add_css_class("running-stopwatch");
            self.time_container.remove_css_class("paused-stopwatch");
            self.time_container.remove_css_class("stopped-stopwatch");
        }

        fn stop(&self) {
            let mut sw = self.timer.get();
            sw.stop();
            self.timer.replace(sw);
            self.state.replace(State::Stopped);

            self.start_btn.set_label("Resume");
            // TODO: Use User's accent colour
            self.start_btn.set_color(Colors::Purple);

            self.clear_btn.set_label("Clear");
            self.clear_btn.set_sensitive(true);
            self.clear_btn.set_color(Colors::Red);

            self.time_container.add_css_class("paused-stopwatch");
            self.time_container.remove_css_class("running-stopwatch");
            self.time_container.remove_css_class("stopped-stopwatch");
        }

        fn clear(&self) {
            let mut sw = self.timer.get();
            sw.reset();
            self.timer.replace(sw);
            self.state.replace(State::Reset);

            self.start_btn.set_label("Start");
            self.start_btn.set_color(Colors::Purple);

            self.clear_btn.set_label("Lap");
            self.clear_btn.set_sensitive(false);
            self.clear_btn.set_color(Colors::Purple);

            self.time_container.add_css_class("stopped-stopwatch");
            self.time_container.remove_css_class("running-stopwatch");
            self.time_container.remove_css_class("paused-stopwatch");
        }

        fn total_laps_duration(&self) -> f64 {
            let mut total = 0.0;
            for i in 0..self.laps.n_items() {
                let lap = self
                    .laps
                    .item(i)
                    .unwrap()
                    .downcast_ref::<StopwatchLap>()
                    .expect("Item should be of type 'StopwatchLap'")
                    .to_owned();

                total += lap.property_value("duration").get::<f64>().unwrap()
            }
            return total;
        }

        fn lap(&self) {
            self.current_lap.replace(self.current_lap.get() + 1);
            let time = self.timer.get().elapsed().as_secs_f64();
            let duration = time - self.total_laps_duration();
            let lap = StopwatchLap::new(duration, self.current_lap.get());
            self.laps.insert(0, &lap);
        }

        pub fn update_time(&self) {
            let duration = Duration::from_std(self.timer.get().elapsed()).unwrap();

            let ms = (duration.num_milliseconds() / 100) % 10;

            self.hours_label
                .set_label(&format!("{}\u{200E}", duration.num_hours()));
            self.minutes_label
                .set_label(&format!("{}\u{200E}", duration.num_minutes()));
            self.seconds_label
                .set_label(&format!("{}\u{200E}", duration.num_seconds()));
            self.miliseconds_label.set_label(&format!("{}", ms));
        }

        #[template_callback]
        fn handle_on_start_btn_click(&self, _button: &Button) {
            debug!("HeFillButton<StopwatchPage>::clicked");
            match self.state.get() {
                State::Reset => self::StopwatchPage::start(self),
                State::Stopped => self::StopwatchPage::start(self),
                State::Running => self::StopwatchPage::stop(self),
            }
        }

        #[template_callback]
        fn handle_on_clear_btn_click(&self, _button: &Button) {
            debug!("HeFillButton<StopwatchPage>::clicked (clear-btn)");
            match self.state.get() {
                State::Stopped => self::StopwatchPage::clear(self),
                State::Running => self::StopwatchPage::lap(self),
                _ => unimplemented!(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StopwatchPage {
        const NAME: &'static str = "NixieStopwatchPage";
        type Type = super::StopwatchPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl BoxImpl for StopwatchPage {}
    impl ObjectImpl for StopwatchPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.timer.replace(Stopwatch::new());

            // TODO move this into its own Rust object
            timeout_add_local(
                std::time::Duration::from_millis(1),
                clone!(@weak obj => @default-return Continue(false), move || {
                    match obj.imp().state.get() {
                        State::Running => obj.imp().update_time(),
                        State::Reset => obj.imp().update_time(),
                        _ => {}
                    }
                    Continue(true)
                }),
            );

            obj.connect_realize(move |_| {
                debug!("GtkBox<StopwatchPage>::realize");
            });
        }
    }

    impl WidgetImpl for StopwatchPage {}
}

use gtk::{
    glib::{self, Object},
    Accessible, Box, Buildable, ConstraintTarget, Widget,
};

glib::wrapper! {
    pub struct StopwatchPage(ObjectSubclass<imp::StopwatchPage>)
        @extends Box, Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl StopwatchPage {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create StopwatchPage")
    }
}