//! GDB TCP server for remote debugging.
//!
//! This module provides a TCP server that accepts GDB connections
//! and handles the RSP protocol.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use log::{error, info, trace, warn};

use crate::{GdbStub, GdbTarget};

/// GDB TCP server configuration.
#[derive(Debug, Clone)]
pub struct GdbServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Host to bind to
    pub host: String,
    /// Maximum connections
    pub max_connections: usize,
}

impl Default for GdbServerConfig {
    fn default() -> Self {
        Self {
            port: 3333,
            host: "127.0.0.1".to_string(),
            max_connections: 1,
        }
    }
}

/// GDB TCP server.
pub struct GdbServer<T: GdbTarget + Send + 'static> {
    /// Configuration
    config: GdbServerConfig,
    /// Target factory function
    target_factory: Arc<dyn Fn() -> T + Send + Sync>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Server thread handle
    thread_handle: Option<JoinHandle<()>>,
}

impl<T: GdbTarget + Send + 'static> GdbServer<T> {
    /// Create a new GDB server.
    pub fn new<F>(config: GdbServerConfig, target_factory: F) -> Self
    where
        F: Into<Arc<dyn Fn() -> T + Send + Sync>>,
    {
        Self {
            config,
            target_factory: target_factory.into(),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Start the server.
    pub fn start(&mut self) -> std::io::Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let config = self.config.clone();
        let target_factory = self.target_factory.clone();

        let handle = thread::spawn(move || {
            let addr = format!("{}:{}", config.host, config.port);
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => {
                    info!("GDB server listening on {}", addr);
                    l
                }
                Err(e) => {
                    error!("Failed to bind GDB server: {}", e);
                    return;
                }
            };

            // Set non-blocking mode for accept
            listener.set_nonblocking(true).ok();

            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("GDB client connected from {}", addr);
                        let target = target_factory();
                        let client_running = running.clone();
                        
                        thread::spawn(move || {
                            handle_client(stream, target, client_running);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection pending, sleep a bit
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(e) => {
                        warn!("Accept error: {}", e);
                    }
                }
            }

            info!("GDB server stopped");
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop the server.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Check if the server is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl<T: GdbTarget + Send + 'static> Drop for GdbServer<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Handle a GDB client connection.
fn handle_client<T: GdbTarget>(mut stream: TcpStream, target: T, running: Arc<AtomicBool>) {
    let mut stub = GdbStub::new(target);
    let mut buffer = [0u8; 4096];
    let mut input_buffer = Vec::new();

    stream.set_nonblocking(false).ok();

    while running.load(Ordering::SeqCst) {
        // Read data from client
        match stream.read(&mut buffer) {
            Ok(0) => {
                info!("GDB client disconnected");
                break;
            }
            Ok(n) => {
                input_buffer.extend_from_slice(&buffer[..n]);
                
                // Process complete packets
                while let Some(packet_end) = find_packet_end(&input_buffer) {
                    let packet_data: Vec<u8> = input_buffer.drain(..=packet_end).collect();
                    let packet_str = String::from_utf8_lossy(&packet_data);
                    
                    trace!("Received: {}", packet_str);
                    
                    // Process the packet
                    let response = stub.process(&packet_str);
                    
                    // Send response
                    if !response.is_empty() {
                        trace!("Sending: {}", response);
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            error!("Write error: {}", e);
                            break;
                        }
                    }
                    
                    // Send ACK (unless in no-ack mode)
                    // Note: GDB stub handles no-ack mode internally
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }
}

/// Find the end of a GDB packet (after checksum).
fn find_packet_end(data: &[u8]) -> Option<usize> {
    // GDB packet format: $data#checksum
    // We need to find the '#' and the two hex digits after it
    
    for i in 0..data.len() {
        if data[i] == b'$' {
            // Found start, look for end
            for j in (i + 1)..data.len() {
                if data[j] == b'#' {
                    // Need 2 more bytes for checksum
                    if j + 2 < data.len() {
                        return Some(j + 2);
                    }
                }
            }
        }
    }
    None
}

/// GDB server builder.
pub struct GdbServerBuilder<T: GdbTarget + Send + 'static> {
    config: GdbServerConfig,
    target_factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: GdbTarget + Send + 'static> GdbServerBuilder<T> {
    /// Create a new builder.
    pub fn new<F>(target_factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            config: GdbServerConfig::default(),
            target_factory: Arc::new(target_factory),
        }
    }

    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set the host.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Set the maximum connections.
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Build the server.
    pub fn build(self) -> GdbServer<T> {
        GdbServer::new(self.config, self.target_factory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GdbError, GdbTarget};
    use crate::target::BreakpointKind;

    struct TestTarget;

    impl GdbTarget for TestTarget {
        fn read_registers(&self) -> Result<Vec<u8>, GdbError> {
            Ok(vec![0; 68])
        }
        fn write_registers(&mut self, _data: &[u8]) -> Result<(), GdbError> {
            Ok(())
        }
        fn read_register(&self, _reg: u32) -> Result<Vec<u8>, GdbError> {
            Ok(vec![0, 0, 0, 0])
        }
        fn write_register(&mut self, _reg: u32, _data: &[u8]) -> Result<(), GdbError> {
            Ok(())
        }
        fn read_memory(&self, _addr: u64, length: u32) -> Result<Vec<u8>, GdbError> {
            Ok(vec![0; length as usize])
        }
        fn write_memory(&mut self, _addr: u64, _data: &[u8]) -> Result<(), GdbError> {
            Ok(())
        }
        fn continue_exec(&mut self) -> Result<(), GdbError> {
            Ok(())
        }
        fn step(&mut self) -> Result<(), GdbError> {
            Ok(())
        }
        fn set_breakpoint(&mut self, _addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
            Ok(())
        }
        fn remove_breakpoint(&mut self, _addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
            Ok(())
        }
        fn is_running(&self) -> bool {
            false
        }
        fn get_pc(&self) -> u64 {
            0
        }
        fn set_pc(&mut self, _pc: u64) -> Result<(), GdbError> {
            Ok(())
        }
        fn get_last_signal(&self) -> u8 {
            5
        }
        fn reset(&mut self) -> Result<(), GdbError> {
            Ok(())
        }
    }

    #[test]
    fn test_server_creation() {
        let server = GdbServerBuilder::new(|| TestTarget)
            .port(3334)
            .build();
        
        assert!(!server.is_running());
    }

    #[test]
    fn test_find_packet_end() {
        let data = b"$g#67";
        assert_eq!(find_packet_end(data), Some(4));
        
        let data = b"$m1000,4#12";
        // '$' at 0, '#' at 8, checksum '12' at 9-10, so end is at index 10
        assert_eq!(find_packet_end(data), Some(10));
        
        let data = b"$g#";
        assert_eq!(find_packet_end(data), None);
    }
}