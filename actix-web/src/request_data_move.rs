use std::{any::type_name, ops::Deref};

use actix_utils::future::{err, ok, Ready};

use crate::{
    dev::Payload, error::ErrorInternalServerError, Error, FromRequest, HttpMessage as _,
    HttpRequest,
};

#[derive(Debug, Clone)]
pub struct ReqDataMove<T: 'static>(T);

impl<T: 'static> ReqDataMove<T> {
    /// Consumes the `ReqDataMove`, returning its wrapped data.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: 'static> Deref for ReqDataMove<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: 'static> FromRequest for ReqDataMove<T> {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(st) = req.extensions_mut().remove::<T>() {
            ok(ReqDataMove(st))
        } else {
            log::debug!(
                "Failed to construct App-level ReqDataMove extractor. \
                 Request path: {:?} (type: {})",
                req.path(),
                type_name::<T>(),
            );
            err(ErrorInternalServerError(
                "Missing expected request extension data",
            ))
        }
    }
}

/* #[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use futures_util::TryFutureExt as _;

    use super::*;
    use crate::{
        dev::Service,
        http::{Method, StatusCode},
        test::{init_service, TestRequest},
        web, App, HttpMessage, HttpResponse,
    };

    #[actix_rt::test]
    async fn req_data_extractor() {
        let srv = init_service(
            App::new()
                .wrap_fn(|req, srv| {
                    if req.method() == Method::POST {
                        req.extensions_mut().insert(42u32);
                    }

                    srv.call(req)
                })
                .service(web::resource("/test").to(
                    |req: HttpRequest, data: Option<ReqDataMove<u32>>| {
                        if req.method() != Method::POST {
                            assert!(data.is_none());
                        }

                        if let Some(data) = data {
                            assert_eq!(*data, 42);
                            assert_eq!(
                                Some(data.into_inner()),
                                req.extensions().get::<u32>().copied()
                            );
                        }

                        HttpResponse::Ok()
                    },
                )),
        )
        .await;

        let req = TestRequest::get().uri("/test").to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = TestRequest::post().uri("/test").to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn req_data_internal_mutability() {
        let srv = init_service(
            App::new()
                .wrap_fn(|req, srv| {
                    let data_before = Rc::new(RefCell::new(42u32));
                    req.extensions_mut().insert(data_before);

                    srv.call(req).map_ok(|res| {
                        {
                            let ext = res.request().extensions();
                            let data_after = ext.get::<Rc<RefCell<u32>>>().unwrap();
                            assert_eq!(*data_after.borrow(), 53u32);
                        }

                        res
                    })
                })
                .default_service(web::to(|data: ReqDataMove<Rc<RefCell<u32>>>| {
                    assert_eq!(*data.borrow(), 42);
                    *data.borrow_mut() += 11;
                    assert_eq!(*data.borrow(), 53);

                    HttpResponse::Ok()
                })),
        )
        .await;

        let req = TestRequest::get().uri("/test").to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
} */
