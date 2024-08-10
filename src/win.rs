use crate::app::KapestrApplication;
use crate::widgets::Post;
use adw::glib::subclass::InitializingObject;
use adw::prelude::*;
use gio::glib::Object;
use gtk4::subclass::prelude::*;

use gtk4::{gio, glib, CompositeTemplate, TemplateChild};

mod imp {
    use adw::subclass::application_window::AdwApplicationWindowImpl;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/py/com/kapestr/main_window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub main_menu_button: TemplateChild<gtk4::MenuButton>,
        #[template_child]
        pub main_stack: TemplateChild<adw::ViewStack>,
        #[template_child(id = "post_template")]
        pub post_template: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "MainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for MainWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Root;

}

impl MainWindow {
    pub fn new(app: &KapestrApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }
    pub fn add_post(&self, author: &str, time: &str, post_content: &str) {
        let post = Post::new();
        post.set_post_data(author, time, post_content);
        self.imp().post_template.add(&post);
    }
}
