use tonic::{transport::Server, Request, Response, Status};

use rfc822::deliverer_server::{Deliverer, DelivererServer};
use rfc822::{Rfc822Request, Rfc822Response};

pub mod rfc822 {
    tonic::include_proto!("rfc822");
}

#[derive(Debug, Default)]
pub struct MyDeliverer {}

#[tonic::async_trait]
impl Deliverer for MyDeliverer {
    async fn deliver(
        &self,
        request: Request<Rfc822Request>,
    ) -> Result<Response<Rfc822Response>, Status> {
        println!("Transaction ID: {}", request.into_inner().transactionid);
        let reply = Rfc822Response {};
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let deliverer = MyDeliverer::default();
    Server::builder()
        .add_service(DelivererServer::new(deliverer))
        .serve(addr)
        .await?;
    Ok(())
}
