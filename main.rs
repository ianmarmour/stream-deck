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
            Event::StreamNode { node_id } => start_streaming(node_id).unwrap(),
            _ => (),
        }
    });

    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();
}

fn start_streaming(id: u32) -> Result<(), Error> {
    gstreamer::init()?;

    let pipeline = gstreamer::parse_launch(&format!("pipewiresrc path={} ! videoconvert ! xvimagesink", id))?;

       // Start playing
       let main_loop = glib::MainLoop::new(None, false);
       let main_loop_clone = main_loop.clone();
       let pipeline_weak = pipeline.downgrade();
       let bus = pipeline.bus().expect("Pipeline has no bus");
       bus.add_watch(move |_, msg| {
           let pipeline = match pipeline_weak.upgrade() {
               Some(pipeline) => pipeline,
               None => return glib::Continue(true),
           };
           let main_loop = &main_loop_clone;
           match msg.view() {
               gstreamer::MessageView::Error(err) => {
                   println!(
                       "Error from {:?}: {} ({:?})",
                       err.src().map(|s| s.path_string()),
                       err.error(),
                       err.debug()
                   );
                   let _ = pipeline.set_state(gstreamer::State::Ready);
                   main_loop.quit();
               }
               gstreamer::MessageView::Eos(..) => {
                   // end-of-stream
                   let _ = pipeline.set_state(gstreamer::State::Ready);
                   main_loop.quit();
               }
               gstreamer::MessageView::ClockLost(_) => {
                   // Get a new clock
                   let _ = pipeline.set_state(gstreamer::State::Paused);
                   let _ = pipeline.set_state(gstreamer::State::Playing);
               }
               _ => (),
           }
           glib::Continue(true)
       })
       .expect("Failed to add bus watch");
   
       main_loop.run();
   
       bus.remove_watch()?;
       pipeline.set_state(gstreamer::State::Null)?;
   
       Ok(())
}