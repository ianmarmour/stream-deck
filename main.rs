use wayland_client::{Display, GlobalManager, global_filter };
use gamescope_protocol::gamescope_pipewire::client::gamescope_pipewire::{GamescopePipewire, Event};

fn main() {
    let wl_gamescope_display_name = "gamescope-0";
    let display = Display::connect_to_name(wl_gamescope_display_name).unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());

    let globals = GlobalManager::new(
        &attached_display, 
    );

    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();

    let gamescope = globals.instantiate_exact::<GamescopePipewire>(1).unwrap();

    gamescope.quick_assign(|_main, ev, _dispatch_data| {
        match ev {
            Event::StreamNode { node_id } => print_gamescope_pipewire_id(node_id),
            _ => println!("{}", "foo")
        }
    });

    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();
}

fn print_gamescope_pipewire_id(id: u32) {
    println!("{}", id)
}

fn build_ui(id: u32) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("First GTK+ Clock");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(260, 40);

    window.show_all();
}