#![feature(unboxed_closures)]

extern crate web_framework;

pub struct CustomRequest<Rq> {
    original: Rq
}

/*pub fn my_middleware<Rq: web_framework::Request, Rp: web_framework::Response>(request: Rq, response: Rp)
    -> (CustomRequest<Rq>, Rp)
{
    (CustomRequest { original: request }, response)
}*/


fn main() {
    let server = web_framework::Server::new();

    let server = server.with(web_framework::to_middleware(|&: rq, rp| {
        (rq, rp)
    }));

    let server = server.with(web_framework::route::get(|&: _, _| {
        use std::io::MemReader;
        //response.set_body(MemReader::new("hello world".as_bytes().to_vec()));
    }));

    server.listen();
}
