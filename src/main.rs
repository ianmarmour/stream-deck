use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, BoxLayout, Orientation, MediaStream};

use gstreamer::prelude::*;
use wayland_client::{Display, GlobalManager, global_filter };
use gamescope_protocol::gamescope_pipewire::client::gamescope_pipewire::{GamescopePipewire, Event};
use anyhow::Error;

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
            Event::StreamNode { node_id } => build_ui(node_id).unwrap(),
            _ => (),
        }
    });

    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();
}


fn build_ui(id: u32) -> Result<(), Error> {
    gstreamer::init()?;
    let pipeline = gstreamer::Pipeline::new(None);

    let src = gstreamer::parse_launch(&format!("pipewiresrc path={} ! videoconvert", id))?;
    let (sink, widget) = if let Ok(gtkglsink) = gstreamer::ElementFactory::make("gtkglsink", None) {
        let glsinkbin = gstreamer::ElementFactory::make("glsinkbin", None).unwrap();
        glsinkbin.set_property("sink", &gtkglsink);
        let widget = gtkglsink.property::<gtk::Widget>("widget");
        (glsinkbin, widget)
    } else {
        let sink = gstreamer::ElementFactory::make("gtksink", None).unwrap();
        let widget = sink.property::<gtk::Widget>("widget");

        (sink, widget)
    };

    pipeline.add_many(&[&src, &sink]).unwrap();
    src.link(&sink).unwrap();

    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    let start_button = gtk::Button::builder()
        .label("Record")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let menu_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let recording_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let button_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();
    
    recording_box.append(&widget);
    button_box.append(&start_button);

    let window = gtk::Window::builder()
        .default_width(1280)
        .default_height(800)
        .title("Hello World")
        .child(&menu_box)
        .child(&recording_box)
        .child(&button_box)
        .build();

    app.add_window(&window);

    app.run();
    Ok(())
}