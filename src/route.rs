use std::ops::Fn;
use Middleware;

pub fn get<Rq, Rp, F: Fn<(Rq, Rp), ()>>(handler: F) -> Route<F> {
    Route {
        handler: handler
    }
}

pub struct Route<H> {
    handler: H
}

impl<Rq, Rp, F: Fn<(Rq, Rp), ()>> Middleware<Rq, Rp, (), ()> for Route<F> {
    fn apply(&self, request: Rq, response: Rp) -> ((), ()) {
        self.handler.call((request, response));
        ((), ())
    }
}
