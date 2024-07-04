use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Orientation, Stack};
use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::cell::RefCell;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::rc::Rc;
use tokio;

struct AppState {
    private_key: RefCell<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new(
        Some("com.example.NostrGTKApp"),
        ApplicationFlags::FLAGS_NONE,
    );
    app.connect_activate(build_initial_ui);
    app.run();
    Ok(())
}

fn build_initial_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Kapestr"));
    window.set_default_size(400, 200);

    let stack = Stack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    let state = Rc::new(AppState {
        private_key: RefCell::new(String::new()),
    });

    let stack_rc = Rc::new(stack);
    build_login_ui(Rc::clone(&stack_rc), Rc::clone(&state));
    build_signin_ui(Rc::clone(&stack_rc));
    build_initial_buttons(Rc::clone(&stack_rc));
    build_send_message_ui(Rc::clone(&stack_rc), Rc::clone(&state));

    window.set_child(Some(&*stack_rc));
    window.show();
}

fn build_initial_buttons(stack: Rc<Stack>) {
    let vbox = GtkBox::new(Orientation::Vertical, 5);
    let login_button = Button::with_label("Log In");
    let signin_button = Button::with_label("Sign In");
    vbox.append(&login_button);
    vbox.append(&signin_button);

    let stack_clone = Rc::clone(&stack);
    login_button.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("login");
    });

    let stack_clone = Rc::clone(&stack);
    signin_button.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("signin");
    });

    stack.add_named(&vbox, Some("initial"));
    stack.set_visible_child_name("initial");
}

fn build_login_ui(stack: Rc<Stack>, state: Rc<AppState>) {
    let vbox = GtkBox::new(Orientation::Vertical, 5);
    let key_entry = Entry::new();
    key_entry.set_placeholder_text(Some("Enter your Nostr private key"));
    vbox.append(&key_entry);
    let login_button = Button::with_label("Log In");
    vbox.append(&login_button);

    let stack_clone = Rc::clone(&stack);
    login_button.connect_clicked(move |_| {
        let private_key = key_entry.text().to_string();
        *state.private_key.borrow_mut() = private_key;
        stack_clone.set_visible_child_name("send_message");
    });

    stack.add_named(&vbox, Some("login"));
}

fn build_signin_ui(stack: Rc<Stack>) {
    let vbox = GtkBox::new(Orientation::Vertical, 5);
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Choose a display name"));
    vbox.append(&name_entry);
    let signin_button = Button::with_label("Sign In");
    vbox.append(&signin_button);

    let stack_clone = Rc::clone(&stack);
    signin_button.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("send_message");
    });

    stack.add_named(&vbox, Some("signin"));
}

fn build_send_message_ui(stack: Rc<Stack>, state: Rc<AppState>) {
    let vbox = GtkBox::new(Orientation::Vertical, 5);
    let message_entry = Entry::new();
    message_entry.set_placeholder_text(Some("Enter your message"));
    vbox.append(&message_entry);
    let send_button = Button::with_label("Send Message");
    vbox.append(&send_button);

    send_button.connect_clicked(move |_| {
        let message = message_entry.text().to_string();
        let private_key = state.private_key.borrow().clone();

        // Spawn a new thread to send the message
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match send_nostr_message(private_key, message).await {
                    Ok(_) => println!("Message sent successfully!"),
                    Err(e) => eprintln!("Error sending message: {}", e),
                }
            });
        });
    });

    stack.add_named(&vbox, Some("send_message"));
}

async fn send_nostr_message(
    private_key: String,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let my_keys = Keys::parse(&private_key)?;
    let event: Event = EventBuilder::text_note(message, []).to_event(&my_keys)?;
    let client = Client::new(&my_keys);
    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay_with_opts(
            "wss://relay.nostr.info",
            RelayOptions::new().proxy(proxy).write(false),
        )
        .await?;
    client
        .add_relay_with_opts(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            RelayOptions::new().proxy(proxy),
        )
        .await?;
    client.connect().await;
    client.send_event(event).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    client.disconnect().await?;
    Ok(())
}
