use gstreamer as gst;

use gtk::prelude::*;
use gst::prelude::*;
use gio::prelude::*;
use glib::prelude::*;

use std::thread;

fn main() {
    gtk::init().unwrap();
    gst::init().unwrap();

    let app = gtk::Application::builder()
        .application_id("org.ianmarmour.StreamDeck")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn get_pipeline() -> gtk::Widget {
    let pipeline = gst::Pipeline::new(None);

    // Pipewire based source for desktop capture
    let src = gst::parse_launch("pipewiresrc").unwrap();

    let (sink, widget) = if let Ok(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
        let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
        glsinkbin.set_property("sink", &gtkglsink);
        let widget = gtkglsink.property::<gtk::Widget>("widget");
        (glsinkbin, widget)
    } else {
        let sink = gst::ElementFactory::make("gtksink", None).unwrap();
        let widget = sink.property::<gtk::Widget>("widget");
        (sink, widget)
    };

    pipeline.add_many(&[&src, &sink]).unwrap();

    src.link(&sink).unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    return widget;
}

fn build_ui(application: &gtk::Application) {
    let video_widget = get_pipeline();

    let start_button = gtk::Button::builder()
        .label("Record")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let recording_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    recording_box.pack_start(&video_widget, true, true, 0);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    button_box.pack_start(&start_button, true, true, 0);

    let total_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    total_box.pack_start(&recording_box, true, true, 0);
    total_box.pack_start(&button_box, true, true, 0);


    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(1280)
        .default_height(800)
        .child(&total_box)
        .title("Hello World")
        .build();
    
    application.add_window(&window);

    window.show_all();
}