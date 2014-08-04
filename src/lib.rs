#![allow(visible_private_types)]
#![feature(unsafe_destructor)]

extern crate http = "tiny-http";

use std::sync::Arc;
use std::sync::atomics::AtomicOption;

pub trait Request {

}

pub trait Response {
    fn set_body<R: Reader + 'static>(&mut self, reader: R);
}

pub trait Middleware<InRq: Request, InRp: Response, OutRq: Request, OutRp: Response> {
    fn apply(&self, request: InRq, response: InRp) -> (OutRq, OutRp);
}

impl<InRq: Request, InRp: Response, OutRq: Request, OutRp: Response>
    Middleware<InRq, InRp, OutRq, OutRp> for fn(InRq, InRp) -> (OutRq, OutRp)
{
    fn apply(&self, request: InRq, response: InRp) -> (OutRq, OutRp) {
        (*self)(request, response)
    }
}

pub trait Handler<InRq: Request, InRp: Response> {
    fn handle(&self, request: InRq, response: InRp);
}

impl<Rq: Request, Rp: Response> Handler<Rq, Rp> for fn(Rq, Rp) {
    fn handle(&self, request: Rq, response: Rp) {
        (*self)(request, response)
    }
}

pub struct Server<Rq, Rp, M> {
    middleware: M,
    routes: Vec<Box<Handler<Rq, Rp> + Send + Share>>,
}

struct EmptyMiddlewareStack;

impl<Rq: Request, Rp: Response> Middleware<Rq, Rp, Rq, Rp> for EmptyMiddlewareStack {
    fn apply(&self, request: Rq, response: Rp) -> (Rq, Rp) {
        (request, response)
    }
}

struct MiddlewareStack<M, N> {
    current: M,
    next: N,
}

impl<InRq: Request, InRp: Response, MidRq: Request, MidRp: Response, OutRq: Request, OutRp: Response,
    Curr: Middleware<InRq, InRp, MidRq, MidRp>, Next: Middleware<MidRq, MidRp, OutRq, OutRp>>
    Middleware<InRq, InRp, OutRq, OutRp> for MiddlewareStack<Curr, Next>
{
    fn apply(&self, request: InRq, response: InRp) -> (OutRq, OutRp) {
        let (r, p) = self.current.apply(request, response);
        self.next.apply(r, p)
    }
}

pub struct BaseRequest {
    request: Arc<AtomicOption<http::Request>>,
}

impl Request for BaseRequest {
}

pub struct BaseResponse {
    response: Option<http::Response<Box<Reader>>>,
    request: Arc<AtomicOption<http::Request>>,
}

impl Response for BaseResponse {
    fn set_body<R: Reader + 'static>(&mut self, reader: R) {
        use std::mem;

        let prev = mem::replace(&mut self.response, None).unwrap();
        let prev = prev.with_data(box reader as Box<Reader>, None);
        self.response = Some(prev);
    }
}

#[unsafe_destructor]
impl Drop for BaseResponse {
    fn drop(&mut self) {
        use std::sync::atomics::Relaxed;
        let request = self.request.take(Relaxed).unwrap();
        request.respond(self.response.take().unwrap());
    }
}

impl Server<BaseRequest, BaseResponse, EmptyMiddlewareStack> {
    pub fn new() -> Server<BaseRequest, BaseResponse, EmptyMiddlewareStack> {
        Server {
            middleware: EmptyMiddlewareStack,
            routes: Vec::new(),
        }
    }
}

impl<Rq: Request, Rp: Response, M> Server<Rq, Rp, M> {
    pub fn get(&mut self, _url: &str, handler: fn(Rq, Rp)) {
        self.routes.push(box handler);
    }
}

impl<OutRq: Request, OutRp: Response,
    CurrM: Middleware<BaseRequest, BaseResponse, OutRq, OutRp> + Send + Share>
    Server<OutRq, OutRp, CurrM>
{
    pub fn with_middleware<NOutRq: Request, NOutRp: Response,
        M: Middleware<OutRq, OutRp, NOutRq, NOutRp>>(self, middleware: M) -> Server<NOutRq, NOutRp, MiddlewareStack<M, CurrM>>
    {
        // cannot add a new middleware if some routes have already been added
        assert!(self.routes.len() == 0);

        Server {
            middleware: MiddlewareStack {
                current: middleware,
                next: self.middleware,
            },
            routes: Vec::new(),
        }
    }

    pub fn with_middleware_fn<NOutRq: Request, NOutRp: Response>(self,
        middleware: fn(OutRq, OutRp) -> (NOutRq, NOutRp)) -> Server<NOutRq, NOutRp, MiddlewareStack<fn(OutRq, OutRp) -> (NOutRq, NOutRp), CurrM>>
    {
        // cannot add a new middleware if some routes have already been added
        assert!(self.routes.len() == 0);

        Server {
            middleware: MiddlewareStack {
                current: middleware,
                next: self.middleware,
            },
            routes: Vec::new(),
        }
    }

    pub fn listen(self) {
        let server = http::Server::new_with_port(1025).unwrap();
        let server = Arc::new(server);

        let middleware = Arc::new(self.middleware);
        let routes = Arc::new(self.routes);

        for _ in range(0u, 4) {
            let server = server.clone();
            let middleware = middleware.clone();
            let routes = routes.clone();

            spawn(proc() {
                loop {
                    use std::io::util::NullReader;

                    let request = server.recv().unwrap();
                    let request = Arc::new(AtomicOption::new(box request));

                    let response = BaseResponse { request: request.clone(), response: Some(http::Response::new(http::StatusCode(200), Vec::new(), box NullReader as Box<Reader>, None, None)) };
                    let request = BaseRequest { request: request };

                    let (request, response) = middleware.apply(request, response);

                    // TODO:
                    let routes = routes.deref();
                    routes[0].handle(request, response);
                }
            })
        }
    }
}
