//! Shows a couple of ways to use the `from_fn` middleware.


use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
  
};
use actix_web_lab::middleware::{ Next, };


pub async fn middleware_wraper(
	mut req: ServiceRequest,
	next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
	//extract head and body
	let (head, _body) = req.parts_mut();

	let token = head.cookie("auth-cookie");
	println!("{:?}",token);
	//get peer ip address

	// macth client and security

	

	

	//create data to pass into the handler
	//pass req to handler
	
	//pass this to handler
	next.call(req).await
}
