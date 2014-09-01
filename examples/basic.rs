#![feature(unboxed_closures)]

extern crate web_framework;

struct JSONResponse<R> {
    response: R,
}

impl<R> JSONResponse<R> {
    fn set_json(self, value: int) {
        // DUMMY
    }
}

fn main() {
    let server = web_framework::Server::new();

    let server = server.with(web_framework::to_middleware(|&: request, response| {
        (request, JSONResponse { response: response })
    }));

    let server = server.with(web_framework::route::get(|&: request, response| {
        response.set_json(5);
    }));

    server.listen();
}
