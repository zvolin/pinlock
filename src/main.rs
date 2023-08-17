use anyhow::Result;
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ConnectionExt, CreateWindowAux, EventMask, GrabMode, InputFocus, Screen, WindowClass,
        },
        Event,
    },
    rust_connection::RustConnection,
    COPY_DEPTH_FROM_PARENT, CURRENT_TIME,
};

struct Window<'connection> {
    id: u32,
    conn: &'connection RustConnection,
}

impl<'connection> Window<'connection> {
    fn create(connection: &'connection RustConnection, screen: &Screen) -> Result<Self> {
        let win = connection.generate_id()?;

        let settings = CreateWindowAux::default()
            .override_redirect(1)
            .background_pixel(31)
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::ENTER_WINDOW
                    | EventMask::LEAVE_WINDOW
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
            );

        // Create the window
        connection.create_window(
            COPY_DEPTH_FROM_PARENT,    // depth (same as root)
            win,                       // window Id
            screen.root,               // parent window
            455,                       // x
            140,                       // y
            1000,                      // width
            800,                       // height
            0,                         // border width
            WindowClass::INPUT_OUTPUT, // class
            screen.root_visual,        // visual
            &settings,
        )?; // masks, not used yet

        // Map the window on the screen
        connection.map_window(win)?;

        connection.flush()?;

        connection.set_input_focus(InputFocus::PARENT, win, CURRENT_TIME)?;
        connection.grab_keyboard(
            true,
            win, //screen.root,
            CURRENT_TIME,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;

        let font = connection.generate_id()?;
        connection.open_font(font, b"cursor")?;

        let cursor = connection.generate_id()?;
        connection.create_glyph_cursor(cursor, font, font, 58, 58 + 1, 0, 0, 0, 0, 0, 0)?;

        connection.grab_pointer(
            true,
            win, //screen.root,
            EventMask::NO_EVENT,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            win,
            cursor,
            CURRENT_TIME,
        )?;

        connection.flush()?;

        Ok(Self {
            id: win,
            conn: connection,
        })
    }
}

impl<'connection> Drop for Window<'connection> {
    fn drop(&mut self) {
        self.conn
            .ungrab_keyboard(CURRENT_TIME)
            .expect("Failed to ungrab the keyboard")
            .check()
            .expect("Keyboard ungrab caused error");
        self.conn
            .ungrab_pointer(CURRENT_TIME)
            .expect("Failed to ungrab the pointer")
            .check()
            .expect("Pointer ungrab caused error");
        self.conn.flush().expect("Failed to send clean up commands");
    }
}

fn main() -> Result<()> {
    // Open the connection to the X server. Use the DISPLAY environment variable.
    let (conn, screen_num) = x11rb::connect(None)?;

    // Get the screen #screen_num
    let screen = &conn.setup().roots[screen_num];

    let _window = Window::create(&conn, screen)?;

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::Expose(event) => {
                println!(
                    "Window {} exposed. Region to be redrawn at location ({},{}) with dimensions \
                     ({},{})",
                    event.window, event.x, event.y, event.width, event.height
                );
            }
            Event::ButtonPress(event) => {
                println!("{:#?}", event.state);
                match event.detail {
                    1 => break Ok(()),
                    4 => println!(
                        "Wheel Button up in window {}, at coordinates ({},{})",
                        event.event, event.event_x, event.event_y
                    ),
                    5 => println!(
                        "Wheel Button down in window {}, at coordinates ({},{})",
                        event.event, event.event_x, event.event_y
                    ),
                    _ => println!(
                        "Button {} pressed in window {}, at coordinates ({},{})",
                        event.detail, event.event, event.event_x, event.event_y
                    ),
                }
            }
            Event::ButtonRelease(event) => {
                println!("{:#?}", event.state);
                println!(
                    "Button {} released in window {}, at coordinates ({},{})",
                    event.detail, event.event, event.event_x, event.event_y
                );
            }
            Event::MotionNotify(event) => {
                println!(
                    "Mouse moved in window {} at coordinates ({},{})",
                    event.event, event.event_x, event.event_y
                );
            }
            Event::EnterNotify(event) => {
                println!(
                    "Mouse entered window {} at coordinates ({},{})",
                    event.event, event.event_x, event.event_y
                );
            }
            Event::LeaveNotify(event) => {
                println!(
                    "Mouse left window {} at coordinates ({},{})",
                    event.event, event.event_x, event.event_y
                );
            }
            Event::KeyPress(event) => {
                println!("{:#?}", event.state);
                println!("Key pressed in window {}", event.event);
            }
            Event::KeyRelease(event) => {
                println!("{:#?}", event.state);
                println!("Key released in window {}", event.event);
            }
            _ => {
                // Unknown event type, ignore it
                println!("Unknown event: {:?}", event);
            }
        }
    }
}
