use std::{collections::HashMap, marker::PhantomData};

pub trait FromRequest {
    // Don't need to bother with error handling.
    fn from_request(req: Request) -> Self;
}

pub struct Request {
    pub body: String,
}

pub trait Handler<T> {
    // We don't need a response type for this demonstration's purpose.
    fn call(&self, req: Request);
}

impl<F> Handler<((),)> for F
where
    F: Fn(),
{
    fn call(&self, _req: Request) {
        self();
    }
}

impl<F, E1> Handler<(E1,)> for F
where
    F: Fn(E1),
    E1: FromRequest,
{
    fn call(&self, req: Request) {
        let part = <E1 as FromRequest>::from_request(req);
        self(part);
    }
}

trait DynHandler {
    fn call(&self, req: Request);
}

struct ErasedHandler<H, T> {
    handler: H,
    _p: PhantomData<T>,
}

impl<H, T> DynHandler for ErasedHandler<H, T>
where
    H: Handler<T>,
    T: 'static,
{
    fn call(&self, req: Request) {
        self.handler.call(req);
    }
}

#[derive(Default)]
pub struct Router {
    routes: HashMap<&'static str, Box<dyn DynHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<H, T>(mut self, pat: &'static str, handler: H) -> Self
    where
        H: Handler<T> + 'static,
        T: 'static,
    {
        self.routes.insert(
            pat,
            Box::new(ErasedHandler {
                handler,
                _p: PhantomData,
            }),
        );
        self
    }

    pub fn run(&self, pat: &'static str, req: Request) {
        self.routes[pat].call(req);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn basic_handlers() {
        static VEC: Mutex<Vec<i32>> = Mutex::new(Vec::new());

        struct MyInt(i32);

        impl FromRequest for MyInt {
            fn from_request(req: Request) -> Self {
                Self(req.body.parse().unwrap())
            }
        }

        fn handler_a() {
            VEC.lock().unwrap().push(1);
        }

        fn handler_b(int: MyInt) {
            VEC.lock().unwrap().push(int.0);
        }

        let router = Router::new().add("/a", handler_a).add("/b", handler_b);

        router.run("/a", Request { body: "".into() });
        router.run("/b", Request { body: "2".into() });

        let vec = VEC.lock().unwrap();
        assert_eq!(*vec, &[1, 2]);
    }
}
