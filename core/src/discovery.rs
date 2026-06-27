use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use crate::{Peer,ToolError};

const BROADCAST_ADDR: &str = "255.255.255.255:58199";
const BIND_ADDR: &str = "0.0.0.0:58199";
const DISCOVERY_MSG: &[u8] = b"TOOLE_DISCOVERY";
