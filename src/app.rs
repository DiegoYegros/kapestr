use std::sync::Arc;

use crate::config::{self, APP_ID, BASE_ID};
use crate::services::PostFetchService;
use crate::win::MainWindow;
use adw::glib::Object;
use adw::prelude::*;
use adw::AboutWindow;
use gio::subclass::prelude::ObjectSubclassIsExt;
use gtk::gio::{self, ActionEntryBuilder, Settings};
use gtk::glib;
use gtk4 as gtk;
use nostr_sdk::{Client, Keys};
use tokio::runtime::Runtime;

mod imp {
    use super::*;
    use adw::glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use adw::subclass::application::AdwApplicationImpl;
    use gtk::subclass::prelude::*;
    use gtk::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct KapestrApplication {
        pub(super) window: OnceCell<MainWindow>,
        pub(super) settings: OnceCell<Settings>,
        pub(super) client: OnceCell<Arc<Client>>,
    }

    #[gtk4::glib::object_subclass]
    impl ObjectSubclass for KapestrApplication {
        const NAME: &'static str = "KapestrApplication";
        type Type = super::KapestrApplication;
        type ParentType = adw::Application;
    }

    impl ApplicationImpl for KapestrApplication {
        fn activate(&self) {
            self.parent_activate();
            let obj = self.obj();
            let window = obj.get_window();
            window.present();
            obj.setup_post_fetch_service();
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            let obj = self.obj();
            obj.set_accels_for_action("win.request", &["<Primary>Return"]);

            let client = Arc::new(Client::new(&Keys::generate()));
            self.client.set(client).expect("Client already set");

            obj.setup_app_actions();
        }
    }

    impl GtkApplicationImpl for KapestrApplication {}
    impl ObjectImpl for KapestrApplication {}
    impl AdwApplicationImpl for KapestrApplication {}
}

gtk4::glib::wrapper! {
    pub struct KapestrApplication(ObjectSubclass<imp::KapestrApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for KapestrApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl KapestrApplication {
    pub fn get() -> Self {
        gio::Application::default()
            .and_downcast::<KapestrApplication>()
            .unwrap()
    }

    pub fn new() -> Self {
        Object::builder().property("application-id", APP_ID).build()
    }

    pub fn get_window(&self) -> &MainWindow {
        let imp = self.imp();
        imp.window.get_or_init(|| MainWindow::new(self))
    }

    pub fn settings(&self) -> &Settings {
        self.imp().settings.get_or_init(|| Settings::new(BASE_ID))
    }

    fn setup_app_actions(&self) {
        let about = ActionEntryBuilder::new("about")
            .activate(|app: &KapestrApplication, _, _| {
                let win = app.get_window();
                let about = AboutWindow::builder()
                    .transient_for(win)
                    .modal(true)
                    .application_name("Kapestr")
                    .application_icon(config::APP_ID)
                    .version(config::VERSION)
                    .website("https://github.com/danirod/Kapestr")
                    .issue_url("https://github.com/danirod/Kapestr/issues")
                    .support_url("https://github.com/danirod/Kapestr/discussions")
                    .developer_name("The Kapestr authors")
                    .copyright("© 2024 the Kapestr authors")
                    .license_type(gtk::License::Gpl30)
                    .build();
                about.present();
            })
            .build();
        self.add_action_entries([about]);
    }

    fn setup_post_fetch_service(&self) {
        let window = self.get_window();
        let client = Arc::clone(self.imp().client.get().expect("Client not initialized"));
        let (service, mut receiver) = PostFetchService::new((*client).clone());

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                service.start().await;
            });
        });

        glib::spawn_future_local(glib::clone!(@weak window => async move {
            while let Some(event) = receiver.recv().await {
                println!("got new event");
                                    PostFetchService::add_post_to_ui(&window, &event);
            }
        }));
    }
}
