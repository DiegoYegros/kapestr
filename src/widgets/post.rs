use adw::subclass::prelude::*;
use gio::glib::Object;
use gtk4::glib;
use gtk4::CompositeTemplate;
use nostr_sdk::Event;
mod imp {
    use super::*;
    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/py/com/kapestr/post.ui")]
    pub struct Post {
        #[template_child]
        pub author_name: TemplateChild<gtk4::Label>,
        #[template_child]
        pub post_time: TemplateChild<gtk4::Label>,
        #[template_child]
        pub content: TemplateChild<gtk4::Label>,
        //     #[template_child]
        //     pub avatar1: TemplateChild<adw::Avatar>,
        //     #[template_child]
        //     pub avatar2: TemplateChild<adw::Avatar>,
    }

    #[adw::glib::object_subclass]
    impl ObjectSubclass for Post {
        const NAME: &'static str = "Post";
        type Type = super::Post;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &adw::glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Post {
        fn constructed(&self) {
            self.parent_constructed();
            self.parent_compute_expand(&mut false, &mut false);
        }
    }
    impl WidgetImpl for Post {}
    impl BoxImpl for Post {}
}

adw::glib::wrapper! {
    pub struct Post(ObjectSubclass<imp::Post>)
        @extends gtk4::Widget, gtk4::Box, @implements gtk4::Buildable;
}

impl Post {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_post_data(&self, author: &str, time: &str, post_content: &str) {
        let imp = imp::Post::from_obj(self);
        imp.author_name.set_label(author);
        imp.post_time.set_label(time);
        imp.content.set_label(post_content);
    }

    pub fn new_from_event(_event: &Event) -> Self {
        todo!()
    }
}
impl Default for Post {
    fn default() -> Self {
        Object::builder().build()
    }
}
