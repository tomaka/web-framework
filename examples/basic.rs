#![feature(unboxed_closures)]

extern crate web_framework;

fn main() {
    let server = web_framework::Server::new();

    let server = server.with(web_framework::to_middleware(|&: request, response| {
        (request, response)
    }));

    let server = server.with(web_framework::route::get(|&: request, response| {
        use std::io::MemReader;
        //response.set_body(MemReader::new("hello world".as_bytes().to_vec()));
    }));

    server.listen();
}
