use std::fmt::Display;
use actix_web::{HttpResponse, error, http::header::ContentType};

#[derive(Debug)]
pub enum ErrorType{
    BADREQUEST(&'static str),
    UNAUTHORIZED(&'static str),
    InternalServerError(&'static str)
}

#[derive(Debug)]
pub struct Error{
    cause:ErrorType
}

impl Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cause {
            ErrorType::BADREQUEST(data)=>{
                write!(f,"{}",data)
            },
            ErrorType::UNAUTHORIZED(data)=>{
                write!(f,"{}",data)
            },
            ErrorType::InternalServerError(data)=>{
                write!(f,"{}",data)
            }
        }
    }
}

impl Error{
    pub fn new(cause:ErrorType)->Self{
        Error { cause  }
    }
}
impl error::ResponseError for Error {
    
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self.cause {
            ErrorType::BADREQUEST(e)=>{return HttpResponse::BadRequest().content_type(ContentType::plaintext()).body(format!("{}",e));},
            ErrorType::UNAUTHORIZED(e)=>{return HttpResponse::Unauthorized().content_type(ContentType::plaintext()).body(format!("{}",e));}
            ErrorType::InternalServerError(e)=>{return HttpResponse::InternalServerError().content_type(ContentType::plaintext()).body(format!("{}",e));}
        }
    }
}