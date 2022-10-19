use gstreamer as gst;
use gst::element_error;
use gst::prelude::*;

use std::env;



fn main() {
    // tutorials_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically)


    multiqueue()
}

fn multiqueue(){
     gst::init().unwrap();

    let args: Vec<_> = env::args().collect();
    let uri: &str;
    let output_file: &str;

    if args.len() == 3 {
        uri = args[1].as_ref();
        output_file = args[2].as_ref();
    } else {
        println!("Usage: multiqueue URI output_file");
        std::process::exit(-1)
    };

    let filesrc = gst::ElementFactory::make("filesrc", None)
        .expect("Could not create filesrc element");

    let qtmux = gst::ElementFactory::make("qtmux", None)
        .expect("Could not create qtmux element");

    let filesink = gst::ElementFactory::make("filesink", None)
        .expect("Could not create filesink element");

    let decodebin = gst::ElementFactory::make("decodebin", None)
        .expect("Could not create decodebin element");

    let x264enc = gst::ElementFactory::make("x264enc", None)
        .expect("Could not create x264enc element");

    let avenc_acc = gst::ElementFactory::make("avenc_aac", None)
        .expect("Could not create avenc_aac element");

    let video_queue = gst::ElementFactory::make("queue", None)
        .expect("Could not create queue element");

    let audio_queue = gst::ElementFactory::make("queue", None)
        .expect("Could not create queue element");

    filesrc.set_property("location", uri);
    filesink.set_property("location", output_file);

    let pipeline = gst::Pipeline::new(None);

    pipeline.add_many(&[&filesrc, &decodebin, &qtmux, &filesink, &x264enc, &avenc_acc, &video_queue, &audio_queue])
        .expect("failed to add elements to pipeline");


    filesrc.link(&decodebin).expect("failed to link filesrc");

    //Link Video
    video_queue.link(&x264enc).expect("Could not link x264enc");
    x264enc.link(&qtmux).expect("Could not link qtmux");

    // Link Audio
    audio_queue.link(&avenc_acc).expect("Could not link avenc_acc");
    avenc_acc.link(&qtmux).expect("Could not link qtmux");

    qtmux.link(&filesink).expect("Could not link filesink");



    decodebin.connect_pad_added(move |demux, src_pad|{
        println!("Received new pad {} from {}",
                 src_pad.name(),
                 demux.name()
        );

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad"); let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure caps");

        let new_pad_type = new_pad_struct.name();


println!("Pad type {}",
                 new_pad_type,
        );

       let new_pad_caps = src_pad
           .current_caps()
           .expect("failed to get caps of new pad");

        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of caps");

        let new_pad_type = new_pad_struct.name();

        if new_pad_type.starts_with("audio"){
            let sink_pad = audio_queue.static_pad("sink")
                .expect("failed to get static sink pad from convert");

            if sink_pad.is_linked() {
                println!("Audio Pad already linked!");
                return;
            }



            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                println!("type of {} link failed: ", new_pad_type);
            } else{
                println!("Linked successfully type {}:", new_pad_type);
            }
        } else if new_pad_type.starts_with("video"){
            let sink_pad = video_queue.static_pad("sink")
                .expect("failed to get static sink pad for queue");

            if sink_pad.is_linked() {
                println!("video pad already linked!");
                return;
            }

            let res = src_pad.link(&sink_pad);
            if res.is_err(){
                println!("type of {} linked failed: ", new_pad_type)
            } else {
                println!("linked succesfully type of {}:", new_pad_type)
            }
        }
    });

    decodebin
        .sync_state_with_parent()
        .expect("Failed to build remux pipeline");



    pipeline.set_state(gst::State::Playing).unwrap();


    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE){
        use gst::MessageView;

        match msg.view() {
            MessageView::Error(err) => {
                println!("Error received from element {:?} {}",
                         err.src().map(|s| s.path_string()),
                         err.error()
                );

                break;
            },

            MessageView::StateChanged(s) => {
                println!(
                    "State changed from {:?}: {:?} -> {:?} ({:?})",
                    s.src().map(|s| s.path_string()),
                    s.old(),
                    s.current(),
                    s.pending()
                );
            },

            MessageView::Eos(_) => break,

            _ => ()
        }
    }

    pipeline.set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the Null state");
}

