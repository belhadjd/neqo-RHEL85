#![allow(unused_variables, dead_code)]

use neqo_transport::connection::Role;
use neqo_transport::frame::StreamType;
use neqo_transport::stream::{Recvable, Sendable};
use neqo_transport::{Error, Res};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum StreamTypeWithRole {
    ClientBiDi,
    ServerBiDi,
    ClientUniDi,
    ServerUniDi,
}

pub fn get_stream_type(r: Role, st: StreamType) -> StreamTypeWithRole {
    if r == Role::Client {
        if st == StreamType::UniDi {
            StreamTypeWithRole::ClientUniDi
        } else {
            StreamTypeWithRole::ClientBiDi
        }
    } else {
        if st == StreamType::UniDi {
            StreamTypeWithRole::ServerUniDi
        } else {
            StreamTypeWithRole::ServerBiDi
        }
    }
}

impl Sendable for Stream {
    /// Enqueue some bytes to send
    fn send(&mut self, buf: &[u8]) -> Res<u64> {
        self.send_buf.extend(buf);
        Ok(buf.len() as u64)
    }

    fn send_data_ready(&self) -> u64 {
        self.send_buf.len() as u64
    }

    fn send_buffer(&mut self) -> &mut VecDeque<u8> {
        &mut self.send_buf_tmp
    }

    fn close(&mut self) {
        self.send_side_closed = true;
    }
}

impl Recvable for Stream {
    fn recv_data_ready(&self) -> u64 {
        self.recv_buf.len() as u64
    }

    /// caller has been told data is available on a stream, and they want to
    /// retrieve it.
    fn read_with_amount(&mut self, buf: &mut [u8], amount: u64) -> Res<u64> {
        assert!(buf.len() >= amount as usize);
        let ret_bytes = std::cmp::min(self.recv_buf.len(), amount as usize);
        let remaining = self.recv_buf.split_off(ret_bytes);
        buf[..ret_bytes].copy_from_slice(&*self.recv_buf);
        self.recv_buf = remaining;
        Ok(ret_bytes as u64)
    }

    fn read(&mut self, buf: &mut [u8]) -> Res<u64> {
        self.read_with_amount(buf, buf.len() as u64)
    }

    fn peek(&mut self, buf: &mut [u8], amount: u64) -> Res<u64> {
        assert!(buf.len() >= amount as usize);
        let ret_bytes = std::cmp::min(self.recv_buf.len(), amount as usize);
        buf[..ret_bytes].copy_from_slice(&self.recv_buf[..ret_bytes]);
        Ok(ret_bytes as u64)
    }

    fn inbound_stream_frame(&mut self, _fin: bool, _offset: u64, _data: Vec<u8>) -> Res<()> {
        Ok(())
    }
    fn needs_flowc_update(&mut self) -> Option<u64> {
        None
    }

    fn stop_sending(&mut self, err: Error) {
        self.stop_sending_error = Some(err);
    }
}

#[derive(Debug)]
pub struct Stream {
    pub send_side_closed: bool,
    pub send_side_stop_sending: bool,
    pub receive_side_closed: bool,
    pub stream_type: StreamTypeWithRole,
    send_buf_tmp: VecDeque<u8>,
    pub send_buf: Vec<u8>,
    pub recv_buf: Vec<u8>,
    pub stop_sending_error: Option<Error>,
    pub error: Option<Error>,
}

impl Stream {
    pub fn new(st: StreamTypeWithRole) -> Stream {
        Stream {
            send_side_closed: false,
            send_side_stop_sending: false,
            receive_side_closed: false,
            stream_type: st,
            send_buf_tmp: VecDeque::new(),
            send_buf: Vec::new(),
            recv_buf: Vec::new(),
            stop_sending_error: None,
            error: None,
        }
    }

    pub fn receive_close(&mut self) {
        self.receive_side_closed = true;
    }

    pub fn reset(&mut self, err: Error) {
        self.error = Some(err);
    }
}
