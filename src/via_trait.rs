use std::collections::HashMap;

pub trait Handler {
    const NAME: &'static str;

    fn exec(self);
}

pub struct Request {
    pub body: String,
}

pub trait FromRequest {
    fn from_request(req: Request) -> Self;
}

pub trait Executor {
    fn exec(&self, req: Request);
}

struct ErasedExecutor<E> {
    executor: E,
}

impl<E> Executor for ErasedExecutor<E>
where
    E: Fn(Request),
{
    fn exec(&self, req: Request) {
        (self.executor)(req);
    }
}

#[derive(Default)]
pub struct Router {
    routes: HashMap<&'static str, Box<dyn Executor>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<H>(mut self) -> Self
    where
        H: Handler + FromRequest + 'static,
    {
        self.routes.insert(
            H::NAME,
            Box::new(ErasedExecutor {
                executor: |req| H::exec(H::from_request(req)),
            }),
        );
        self
    }

    pub fn run(&self, pat: &'static str, req: Request) {
        self.routes[pat].exec(req);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn basic_handlers() {
        static VEC: Mutex<Vec<i32>> = Mutex::new(Vec::new());

        struct A;
        impl FromRequest for A {
            fn from_request(_req: Request) -> Self {
                Self
            }
        }
        impl Handler for A {
            const NAME: &'static str = "a";

            fn exec(self) {
                VEC.lock().unwrap().push(1);
            }
        }

        struct B(i32);
        impl FromRequest for B {
            fn from_request(req: Request) -> Self {
                Self(req.body.parse().unwrap())
            }
        }
        impl Handler for B {
            const NAME: &'static str = "b";

            fn exec(self) {
                VEC.lock().unwrap().push(self.0);
            }
        }

        let router = Router::new().register::<A>().register::<B>();

        router.run("a", Request { body: "".into() });
        router.run("b", Request { body: "2".into() });

        let vec = VEC.lock().unwrap();
        assert_eq!(*vec, &[1, 2]);
    }
}
