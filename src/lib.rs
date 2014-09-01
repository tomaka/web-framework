#![allow(visible_private_types)]
#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]

extern crate tiny_http;

use std::ops::Fn;
use std::sync::Arc;
use std::sync::atomics::AtomicOption;

pub mod route;

/// TODO: use associated types
pub trait Middleware<InRq, InRp, OutRq, OutRp> {
    fn apply(&self, request: InRq, response: InRp) -> (OutRq, OutRp);
}

pub struct FnToMiddleware<F>(F);

pub fn to_middleware<F>(f: F) -> FnToMiddleware<F> {
    FnToMiddleware(f)
}

impl<InRq, InRp, OutRq, OutRp, F: Fn<(InRq, InRp), (OutRq, OutRp)>> Middleware<InRq, InRp, OutRq, OutRp> for FnToMiddleware<F> {
    fn apply(&self, request: InRq, response: InRp) -> (OutRq, OutRp) {
        let &FnToMiddleware(ref me) = self;
        me.call((request, response))
    }
}

pub struct Server<'a, Rq, Rp> {
    middleware: Box<Fn<(BaseRequest, BaseResponse), (Rq, Rp)> + 'a>,
}

pub struct BaseRequest {
    request: Arc<AtomicOption<tiny_http::Request>>,
}

pub struct BaseResponse {
    //response: Option<tiny_http::Response<Box<Reader + Send>>>,
    request: Arc<AtomicOption<tiny_http::Request>>,
}

impl BaseResponse {
    fn set_body<R: Reader + Send>(&mut self, reader: R) {
        /*use std::mem;

        let prev = mem::replace(&mut self.response, None).unwrap();
        let prev = prev.with_data(box reader as Box<Reader + Send>, None);
        self.response = Some(prev);*/
    }
}

#[unsafe_destructor]
impl Drop for BaseResponse {
    fn drop(&mut self) {
        /*use std::sync::atomics::Relaxed;
        let request = self.request.take(Relaxed).unwrap();
        request.respond(self.response.take().unwrap());*/
    }
}

impl Server<'static, BaseRequest, BaseResponse> {
    pub fn new() -> Server<'static, BaseRequest, BaseResponse> {
        let middleware = |&: rq, rp| (rq, rp);
        let middleware: Box<Fn<(BaseRequest, BaseResponse), (BaseRequest, BaseResponse)> + 'static> = box middleware;

        Server {
            middleware: middleware,
        }
    }
}

impl<'a, Rq, Rp> Server<'a, Rq, Rp> {
    pub fn with<'b: 'a, OutRq, OutRp, M: Middleware<Rq, Rp, OutRq, OutRp> + 'b>(self, middleware: M)
        -> Server<'b, OutRq, OutRp>
    {
        let current = self.middleware;

        let middleware = |&: rq, rp| {
            let (rq, rp) = current.call((rq, rp));
            middleware.apply(rq, rp)
        };

        let middleware: Box<Fn<(BaseRequest, BaseResponse), (OutRq, OutRp)> + 'b> = box middleware;

        Server {
            middleware: middleware,
        }
    }
}

impl<'a> Server<'a, (), ()> {
    pub fn listen(self) {
        let server = tiny_http::ServerBuilder::new().with_port(1025).build().unwrap();
        let server = Arc::new(server);

        /*let middleware = Arc::new(self.middleware);

        for _ in range(0u, 4) {
            let server = server.clone();
            let middleware = middleware.clone();
            let routes = routes.clone();

            spawn(proc() {*/
                loop {
                    use std::io::util::NullReader;

                    let request = server.recv().unwrap();
                    let request = Arc::new(AtomicOption::new(box request));

                    let response = BaseResponse { request: request.clone()/*, response: Some(tiny_http::Response::new(tiny_http::StatusCode(200), Vec::new(), box NullReader as Box<Reader>, None, None))*/ };
                    let request = BaseRequest { request: request };

                    self.middleware.call((request, response));

                    // TODO:
                    //let routes = routes.deref();
                    //routes[0].handle(request, response);
                }
            //})
        //}
    }
}
