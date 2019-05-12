#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStateReq {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoopState {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(enumeration="RecordMode", tag="2")]
    pub record_mode: i32,
    #[prost(enumeration="PlayMode", tag="3")]
    pub play_mode: i32,
    #[prost(int64, tag="4")]
    pub time: i64,
    #[prost(int64, tag="5")]
    pub length: i64,
    #[prost(bool, tag="6")]
    pub active: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct State {
    #[prost(message, repeated, tag="1")]
    pub loops: ::std::vec::Vec<LoopState>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Command {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RecordMode {
    None = 0,
    Ready = 1,
    Record = 2,
    Overdub = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PlayMode {
    Paused = 0,
    Playing = 1,
}
pub mod client {
    use ::tower_grpc::codegen::client::*;
    use super::{GetStateReq, State};

    #[derive(Debug, Clone)]
    pub struct Looper<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> Looper<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        pub fn get_state<R>(&mut self, request: grpc::Request<GetStateReq>) -> grpc::server_streaming::ResponseFuture<State, T::Future>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<GetStateReq>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/protos.Looper/GetState");
            self.inner.server_streaming(request, path)
        }
    }
}

pub mod server {
    use ::tower_grpc::codegen::server::*;
    use super::{GetStateReq, State};

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    pub trait Looper: Clone {
        type GetStateStream: futures::Stream<Item = State, Error = grpc::Status>;
        type GetStateFuture: futures::Future<Item = grpc::Response<Self::GetStateStream>, Error = grpc::Status>;

        fn get_state(&mut self, request: grpc::Request<GetStateReq>) -> Self::GetStateFuture;
    }

    #[derive(Debug, Clone)]
    pub struct LooperServer<T> {
        looper: T,
    }

    impl<T> LooperServer<T>
    where T: Looper,
    {
        pub fn new(looper: T) -> Self {
            Self { looper }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for LooperServer<T>
    where T: Looper,
    {
        type Response = http::Response<looper::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = looper::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::looper::Kind::*;

            match request.uri().path() {
                "/protos.Looper/GetState" => {
                    let service = looper::methods::GetState(self.looper.clone());
                    let response = grpc::server_streaming(service, request);
                    looper::ResponseFuture { kind: GetState(response) }
                }
                _ => {
                    looper::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for LooperServer<T>
    where T: Looper,
    {
        type Response = Self;
        type Error = grpc::Never;
        type Future = futures::FutureResult<Self::Response, Self::Error>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(futures::Async::Ready(()))
        }

        fn call(&mut self, _target: ()) -> Self::Future {
            futures::ok(self.clone())
        }
    }

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for LooperServer<T>
    where T: Looper,
    {
        type Response = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Response;
        type Error = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Error;
        type Future = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Future;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            tower::Service::<http::Request<grpc::BoxBody>>::poll_ready(self)
        }

        fn call(&mut self, request: http::Request<tower_hyper::Body>) -> Self::Future {
            let request = request.map(|b| grpc::BoxBody::map_from(b));
            tower::Service::<http::Request<grpc::BoxBody>>::call(self, request)
        }
    }

    pub mod looper {
        use ::tower_grpc::codegen::server::*;
        use super::Looper;
        use super::super::GetStateReq;

        pub struct ResponseFuture<T>
        where T: Looper,
        {
            pub(super) kind: Kind<
                // GetState
                grpc::server_streaming::ResponseFuture<methods::GetState<T>, grpc::BoxBody, GetStateReq>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: Looper,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: GetState(body) }
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: __Generated__Unimplemented(body) }
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where T: Looper,
        {
            pub(super) kind: Kind<
                // GetState
                grpc::Encode<<methods::GetState<T> as grpc::ServerStreamingService<GetStateReq>>::ResponseStream>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: Looper,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    GetState(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<GetState, __Generated__Unimplemented> {
            GetState(GetState),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{Looper, GetStateReq};

            pub struct GetState<T>(pub T);

            impl<T> tower::Service<grpc::Request<GetStateReq>> for GetState<T>
            where T: Looper,
            {
                type Response = grpc::Response<T::GetStateStream>;
                type Error = grpc::Status;
                type Future = T::GetStateFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<GetStateReq>) -> Self::Future {
                    self.0.get_state(request)
                }
            }
        }
    }
}