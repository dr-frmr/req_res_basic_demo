use kinode_process_lib::{println, *};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

call_init!(init);

fn init(our: Address) {
    println!("p1: start");

    let p2 = Address::new(our.node(), ("p2", our.package(), our.publisher()));

    // fire off a request without expecting any response
    Request::to(&p2).body(b"hello").send().unwrap();

    // fire off a request and wait 1 second for a response
    // waiting for a response will block in place.
    // p2 is programmed to wait 2 seconds before responding, so this will time out
    let maybe_message = Request::to(&p2)
        .body(b"please respond")
        .send_and_await_response(1)
        .unwrap();
    println!("p1: maybe_message: {}", maybe_message.is_ok());

    // send that request again, but wait long enough to get the response
    let definitely_message = Request::to(&p2)
        .body(b"please respond")
        .send_and_await_response(5)
        .unwrap();
    println!("p1: definitely_message: {}", definitely_message.is_ok());

    // send a request with a blob of data, wait for a response, and expect no blob
    let _ = Request::to(&p2)
        .body(b"please respond")
        .blob_bytes(vec![1, 2, 3, 4, 5])
        .send_and_await_response(5)
        .unwrap();
    // call get_blob to access the optional lazy-load payload from the response
    assert!(get_blob().is_none());
    println!("p1: got no blob");

    // send a request with a blob, wait for a response, and expect the same blob
    let _ = Request::to(&p2)
        .body(b"please respond with blob")
        .blob_bytes(vec![1, 2, 3, 4, 5])
        .send_and_await_response(5)
        .unwrap();
    assert_eq!(vec![1, 2, 3, 4, 5], get_blob().unwrap().bytes);
    println!("p1: got blob!");

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = Request::to(&p2)
            .body(b"please ignore this blob")
            .send_and_await_response(1)
            .unwrap()
            .unwrap();
    }
    println!(
        "p1: 100 requests with *no* blob took: {:?}",
        start.elapsed()
    );

    // now, to see the performance savings of blobbing, we will send
    // 100 requests with a 100MB blob and measure the time it takes
    // to send and receive all of them. first, p2 will be programmed
    // to "get" each blob, and next, p2 will be programmed to ignore it.
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = Request::to(&p2)
            .body(b"please grab this blob")
            .blob_bytes(vec![0; 100_000_000])
            .send_and_await_response(1)
            .unwrap()
            .unwrap();
    }
    println!(
        "p1: 100 requests with 100MB blob took: {:?}",
        start.elapsed()
    );

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = Request::to(&p2)
            .body(b"please ignore this blob")
            .blob_bytes(vec![0; 100_000_000])
            .send_and_await_response(1)
            .unwrap()
            .unwrap();
    }
    println!(
        "p1: 100 requests with 100MB blob (ignored) took: {:?}",
        start.elapsed()
    );

    // send a request that expects a response, but don't wait for it.
    // instead, wait for an incoming request and respond to it.
    // p2 will use this to figure out what data to respond to us with.
    // (can imagine this playing out across n processes -- chains freely)
    let my_number = b"5";
    Request::to(&p2)
        .body(b"i'm looking for a number between one and ten...")
        .expects_response(5)
        .send()
        .unwrap();

    let message = await_message().unwrap();
    if message.source() == &p2 && message.is_request() {
        // give them the number we want
        Response::new().body(my_number).send().unwrap();
    }

    // now, handle response from first request
    let message = await_message().unwrap();
    assert_eq!(my_number, message.body());
    println!("p1: got the number we wanted!");

    println!("p1: end");
}
