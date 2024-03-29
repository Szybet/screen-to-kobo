// Logging
use log::{debug, info};

// Network
use crate::api::{FromClientMessage, FromServerMessage};
use message_io::network::NetEvent;
use message_io::node::{NodeHandler, NodeListener};

// Threads
use std::sync::mpsc::Sender;
use crate::ThreadCom;
use std::sync::Arc;

pub fn run(handler: Arc<NodeHandler<()>>, listener: NodeListener<()>, tx_to_gui: Sender<ThreadCom>) {
    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (),
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event
            info!("Client ({}) connected", endpoint.addr());
            tx_to_gui.send(ThreadCom::ClientConnected(endpoint, _listener_id)).unwrap();
        }
        NetEvent::Message(endpoint, input_data) => {
            debug!("Received raw input data with length: {}", input_data.len());
            let message: FromClientMessage = bincode::deserialize(input_data).unwrap();
            match message {
                FromClientMessage::Ping => {
                    info!("Received Ping from client");
                    let output_data = bincode::serialize(&FromServerMessage::Pong).unwrap();
                    info!("Sending Pong");
                    handler.network().send(endpoint, &output_data);
                }
                FromClientMessage::Click(x, y) => {
                    debug!("Received click from client");
                }
            }
        }
        NetEvent::Disconnected(endpoint) => {
            info!("Client ({}) disconnected", endpoint.addr(),);
        }
    });
}
