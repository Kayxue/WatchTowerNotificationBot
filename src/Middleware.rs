use xitca_web::{
    WebContext,
    error::Error,
    handler::Responder,
    http::{StatusCode, WebResponse},
    service::Service,
};

use crate::CustomError::{BadRequest, InternalServerError};

pub async fn error_handler<S, C>(s: &S, mut ctx: WebContext<'_, C>) -> Result<WebResponse, Error>
where
    C: 'static,
    S: for<'r> Service<WebContext<'r, C>, Response = WebResponse, Error = Error>,
{
    match s.call(ctx.reborrow()).await {
        Ok(res) => Ok(res),
        Err(e) => {
            // generate http response actively. from here it's OK to early return it in Result::Ok
            // variant as error_handler function's output
            let _res = e.call(ctx.reborrow()).await?;
            // return Ok(_res);

            // upcast trait and downcast to concrete type again.
            // this offers the ability to regain typed error specific error handling.
            // *. this is a runtime feature and not reinforced at compile time.
            if let Some(_e) = e.upcast().downcast_ref::<BadRequest>() {
                return (_e.message.to_owned(), StatusCode::BAD_REQUEST)
                    .respond(ctx)
                    .await;
            }

            if let Some(_e) = e.upcast().downcast_ref::<InternalServerError>() {
                return (_e.message.to_owned(), StatusCode::INTERNAL_SERVER_ERROR)
                    .respond(ctx)
                    .await;
            }

            // the most basic error handling is to ignore it and return as is. xitca-web is able to take care
            // of error by utilizing it's according trait implements(Debug,Display,Error and Service impls)
            Err(e)
        }
    }
}
