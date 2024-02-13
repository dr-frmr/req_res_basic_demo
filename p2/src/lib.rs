use kinode_process_lib::{println, *};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

fn handle_message(our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    match message.body() {
        b"please respond" => {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            Response::new().body(b"hai").send().unwrap();
        }
        b"please respond with blob" => {
            // we inherit the blob from the request, the bytes never need to be ingested
            Response::new().body(b"hai").inherit(true).send().unwrap();
        }
        b"please grab this blob" => {
            // we will grab the blob from the request
            let blob = message.blob().unwrap();
            Response::new().body(b"hai").inherit(true).send().unwrap();
        }
        b"please ignore this blob" => {
            // we will ignore the blob from the request
            Response::new().body(b"hai").inherit(true).send().unwrap();
        }
        b"i'm looking for a number between one and ten..." => {
            // figure out what number to respond with by asking p1
            let p1 = message.source();
            let res = Request::to(p1)
                .body(b"what number should i respond with?")
                .send_and_await_response(5)
                .unwrap()
                .unwrap();
            Response::new().body(res.body()).send().unwrap();
        }
        _ => {}
    }

    Ok(())
}

call_init!(init);

fn init(our: Address) {
    println!("p2: start");

    loop {
        match handle_message(&our) {
            Ok(()) => continue,
            Err(e) => {
                println!("req_res_tester: error: {:?}", e);
                break;
            }
        };
    }

    println!("p2: end");
}
