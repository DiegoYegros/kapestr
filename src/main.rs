use async_utility::futures_util::StreamExt;
use gtk4::gio::{ApplicationFlags, SimpleAction};
use gtk4::glib::{self, clone};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox, MenuButton,
    Orientation, ScrolledWindow,
};
use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

struct AppState {
    private_key: Mutex<String>,
    client: Mutex<Option<Client>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new(Some("py.com.kapestr"), ApplicationFlags::FLAGS_NONE);
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Kapestr"));
    window.set_default_size(800, 600);

    let state = Arc::new(AppState {
        private_key: Mutex::new(String::new()),
        client: Mutex::new(None),
    });

    let main_box = GtkBox::new(Orientation::Horizontal, 0);

    // Left side: Menu and input
    let left_box = GtkBox::new(Orientation::Vertical, 5);
    left_box.set_width_request(200);

    // Hamburger menu
    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    build_menu(&menu_button, app, Arc::clone(&state));
    left_box.append(&menu_button);

    // Message input
    let message_entry = Entry::new();
    message_entry.set_placeholder_text(Some("Enter your message"));
    left_box.append(&message_entry);

    let send_button = Button::with_label("Send Message");
    left_box.append(&send_button);

    send_button.connect_clicked(clone!(@strong state, @strong message_entry => move |_| {
        let message = message_entry.text().to_string();
        let private_key = state.private_key.lock().unwrap().clone();

        glib::MainContext::default().spawn_local(clone!(@strong message_entry => async move {
            match send_nostr_message(private_key, message).await {
                Ok(_) => {
                    println!("Message sent successfully!");
                    message_entry.set_text("");
                },
                Err(e) => eprintln!("Error sending message: {}", e),
            }
        }));
    }));

    main_box.append(&left_box);

    // Right side: Post feed
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_hexpand(true);
    scrolled_window.set_vexpand(true);

    let posts_list = ListBox::new();
    posts_list.set_selection_mode(gtk4::SelectionMode::None);
    scrolled_window.set_child(Some(&posts_list));

    main_box.append(&scrolled_window);

    window.set_child(Some(&main_box));

    // Start fetching posts
    let (tx, rx) = mpsc::channel(100);
    fetch_posts(Arc::clone(&state), tx);
    receive_posts(posts_list, rx);

    window.show();
}

fn build_menu(menu_button: &MenuButton, app: &Application, state: Arc<AppState>) {
    let menu = gio::Menu::new();
    let login_action = SimpleAction::new("login", None);
    let signin_action = SimpleAction::new("signin", None);

    login_action.connect_activate(clone!(@strong state => move |_, _| {
        show_login_dialog(Arc::clone(&state));
    }));

    signin_action.connect_activate(clone!(@strong state => move |_, _| {
        show_signin_dialog(Arc::clone(&state));
    }));

    app.add_action(&login_action);
    app.add_action(&signin_action);

    menu.append(Some("Login"), Some("app.login"));
    menu.append(Some("Sign In"), Some("app.signin"));

    menu_button.set_menu_model(Some(&menu));
}

fn show_login_dialog(state: Arc<AppState>) {
    let dialog = gtk4::Dialog::with_buttons(
        Some("Login"),
        None::<&ApplicationWindow>,
        gtk4::DialogFlags::MODAL,
        &[
            ("OK", gtk4::ResponseType::Ok),
            ("Cancel", gtk4::ResponseType::Cancel),
        ],
    );

    let content_area = dialog.content_area();
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Enter your private key"));
    content_area.append(&entry);

    dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Ok {
            let private_key = entry.text().to_string();
            *state.private_key.lock().unwrap() = private_key;
            println!("Logged in successfully!");
        }
        dialog.close();
    });

    dialog.show();
}

fn show_signin_dialog(state: Arc<AppState>) {
    let dialog = gtk4::Dialog::with_buttons(
        Some("Sign In"),
        None::<&ApplicationWindow>,
        gtk4::DialogFlags::MODAL,
        &[
            ("OK", gtk4::ResponseType::Ok),
            ("Cancel", gtk4::ResponseType::Cancel),
        ],
    );

    let content_area = dialog.content_area();
    let label = Label::new(Some("Generating a new private key for you..."));
    content_area.append(&label);

    dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Ok {
            let keys = Keys::generate();
            let private_key = keys.secret_key().unwrap().display_secret().to_string();
            *state.private_key.lock().unwrap() = private_key.clone();
            println!("New account created. Your private key is: {}", private_key);
        }
        dialog.close();
    });

    dialog.show();
}

fn fetch_posts(state: Arc<AppState>, tx: mpsc::Sender<Event>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = Client::new(&Keys::generate());
            client.add_relay("wss://relay.damus.io").await.unwrap();
            client.connect().await;

            *state.client.lock().unwrap() = Some(client.clone());

            let subscription = client
                .subscribe(vec![Filter::new().kind(Kind::TextNote).limit(20)], None)
                .await;

            let mut events = client.notifications();
            while let Ok(notification) = events.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    let _ = tx.send(*event).await;
                }
            }

            client.unsubscribe(subscription).await;
        });
    });
}

fn receive_posts(posts_list: ListBox, mut rx: mpsc::Receiver<Event>) {
    glib::MainContext::default().spawn_local(async move {
        while let Some(event) = rx.recv().await {
            let row = GtkBox::new(Orientation::Vertical, 5);
            let author = Label::new(Some(&format!("Author: {}", event.pubkey)));
            let content = Label::new(Some(&event.content));
            content.set_wrap(true);
            row.append(&author);
            row.append(&content);
            posts_list.insert(&row, -1);
        }
    });
}

async fn send_nostr_message(
    private_key: String,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let my_keys = Keys::parse(&private_key)?;
    let event: Event = EventBuilder::text_note(message, Vec::new()).to_event(&my_keys)?;
    let client = Client::new(&my_keys);
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;
    client.send_event(event).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    client.disconnect().await?;
    Ok(())
}
