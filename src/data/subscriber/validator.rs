// use futures::StreamExt;

// use crate::{
//     data::{exchange_connector::Connector, protocols::ws::WsRead},
//     error::SocketError,
// };

// pub struct WebSocketValidator;

// impl WebSocketValidator {
//     pub async fn validate<ExchangeConnector>(ws: &mut WsRead, connector: &ExchangeConnector) -> Result<ExchangeConnector, SocketError>
//     where
//         ExchangeConnector: Connector + Send,
//     {
//         let time_out = ExchangeConnector::subscription_timeout();
//         let mut subscriptions_sucessful = false;

//         loop {
//             if subscriptions_sucessful {
//                 break Err(SocketError::Subscribe(format!("Subscription validation failed")))
//             }

//             tokio::select! {
//              _ = tokio::time::sleep(time_out) => {
//                  break Err(SocketError::Subscribe(format!("Subscription validation failed")))
//              }

//             message = ws.next() => {
//                 let response = match message {
//                     Some(response) => reponse,
//                     None = break Err(SocketError::Subscribe(String::from("WebSocket stream terminated unexpectedly")))
//                 }

//                 match
            
//             }
//             }
//         }
//     }
// }
