//! MCP Transport Layer
//!
//! Handles communication with MCP servers via stdio and SSE transports.

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError, error_codes};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// Transport trait for MCP communication
#[async_trait::async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a JSON-RPC request and wait for response
    async fn send_request(&mut self, request: crate::protocol::JsonRpcRequest) -> Result<crate::protocol::JsonRpcResponse, TransportError>;
    
    /// Send a JSON-RPC notification (no response expected)
    async fn send_notification(&mut self, notification: crate::protocol::JsonRpcNotification) -> Result<(), TransportError>;
    
    /// Receive next message
    async fn receive_message(&mut self) -> Result<Option<Value>, TransportError>;
    
    /// Close the transport
    async fn close(&mut self) -> Result<(), TransportError>;
    
    /// Check if transport is connected
    fn is_connected(&self) -> bool;
}

/// Transport errors
#[derive(Debug, Clone)]
pub enum TransportError {
    Io(String),
    Json(String),
    Timeout,
    Disconnected,
    Protocol(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Io(e) => write!(f, "IO error: {}", e),
            TransportError::Json(e) => write!(f, "JSON error: {}", e),
            TransportError::Timeout => write!(f, "Operation timed out"),
            TransportError::Disconnected => write!(f, "Transport disconnected"),
            TransportError::Protocol(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

impl std::error::Error for TransportError {}

/// Stdio transport implementation
pub struct StdioTransport {
    /// Child process
    child: Child,
    /// Reader for stdout
    stdout_reader: BufReader<tokio::process::ChildStdout>,
    /// Writer for stdin
    stdin: tokio::process::ChildStdin,
    /// Connected flag
    connected: bool,
}

impl StdioTransport {
    /// Create a new stdio transport by spawning an MCP server process
    pub async fn new(command: impl AsRef<str>, args: &[String]) -> Result<Self, TransportError> {
        let mut child = Command::new(command.as_ref())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TransportError::Io(e.to_string()))?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| TransportError::Io("Failed to capture stdout".to_string()))?;
        let stdin = child.stdin.take()
            .ok_or_else(|| TransportError::Io("Failed to capture stdin".to_string()))?;
        
        Ok(Self {
            child,
            stdout_reader: BufReader::new(stdout),
            stdin,
            connected: true,
        })
    }
    
    /// Read a line from stdout
    async fn read_line(&mut self) -> Result<Option<String>, TransportError> {
        let mut line = String::new();
        match self.stdout_reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(line)),
            Err(e) => Err(TransportError::Io(e.to_string())),
        }
    }
    
    /// Write a line to stdin
    async fn write_line(&mut self, line: impl AsRef<[u8]>) -> Result<(), TransportError> {
        self.stdin.write_all(line.as_ref()).await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        self.stdin.write_all(b"\n").await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        self.stdin.flush().await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl McpTransport for StdioTransport {
    async fn send_request(&mut self, request: crate::protocol::JsonRpcRequest) -> Result<crate::protocol::JsonRpcResponse, TransportError> {
        if !self.connected {
            return Err(TransportError::Disconnected);
        }
        
        let json = serde_json::to_string(&request)
            .map_err(|e| TransportError::Json(e.to_string()))?;
        
        self.write_line(&json).await?;
        
        // Wait for response with matching ID
        let request_id = request.id.clone();
        let timeout = tokio::time::Duration::from_secs(30);
        let deadline = tokio::time::Instant::now() + timeout;
        
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(TransportError::Timeout);
            }
            
            let line = tokio::time::timeout(remaining, self.read_line()).await
                .map_err(|_| TransportError::Timeout)?;
            
            if let Some(line) = line? {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                let value: Value = serde_json::from_str(trimmed)
                    .map_err(|e| TransportError::Json(e.to_string()))?;
                
                // Check if it's a response with matching ID
                if let Some(id) = value.get("id") {
                    let expected_id = request_id.as_ref().unwrap_or(&Value::Null);
                    if id == expected_id {
                        let response: crate::protocol::JsonRpcResponse = serde_json::from_value(value)
                            .map_err(|e| TransportError::Json(e.to_string()))?;
                        return Ok(response);
                    }
                }
                // Otherwise it's a notification or unsolicited message, skip for now
            } else {
                return Err(TransportError::Disconnected);
            }
        }
    }
    
    async fn send_notification(&mut self, notification: crate::protocol::JsonRpcNotification) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::Disconnected);
        }
        
        let json = serde_json::to_string(&notification)
            .map_err(|e| TransportError::Json(e.to_string()))?;
        
        self.write_line(&json).await
    }
    
    async fn receive_message(&mut self) -> Result<Option<Value>, TransportError> {
        if !self.connected {
            return Ok(None);
        }
        
        let line = self.read_line().await?;
        if let Some(line) = line {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            
            let value: Value = serde_json::from_str(trimmed)
                .map_err(|e| TransportError::Json(e.to_string()))?;
            Ok(Some(value))
        } else {
            self.connected = false;
            Ok(None)
        }
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        self.connected = false;
        let _ = self.child.kill().await;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// In-memory transport for testing and embedded servers
pub struct MemoryTransport {
    /// Sender for outgoing messages
    outgoing: mpsc::UnboundedSender<Value>,
    /// Receiver for incoming messages
    incoming: mpsc::UnboundedReceiver<Value>,
    /// Connected flag
    connected: bool,
}

impl MemoryTransport {
    /// Create a new in-memory transport pair
    pub fn create_pair() -> (Self, Self) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();
        
        let transport1 = Self {
            outgoing: tx1,
            incoming: rx2,
            connected: true,
        };
        
        let transport2 = Self {
            outgoing: tx2,
            incoming: rx1,
            connected: true,
        };
        
        (transport1, transport2)
    }
}

#[async_trait::async_trait]
impl McpTransport for MemoryTransport {
    async fn send_request(&mut self, request: crate::protocol::JsonRpcRequest) -> Result<crate::protocol::JsonRpcResponse, TransportError> {
        if !self.connected {
            return Err(TransportError::Disconnected);
        }
        
        let value = serde_json::to_value(&request)
            .map_err(|e| TransportError::Json(e.to_string()))?;
        
        self.outgoing.send(value)
            .map_err(|_| TransportError::Disconnected)?;
        
        // Wait for response
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), self.incoming.recv()).await {
            Ok(Some(response)) => {
                let resp: crate::protocol::JsonRpcResponse = serde_json::from_value(response)
                    .map_err(|e| TransportError::Json(e.to_string()))?;
                Ok(resp)
            }
            Ok(None) => Err(TransportError::Disconnected),
            Err(_) => Err(TransportError::Timeout),
        }
    }
    
    async fn send_notification(&mut self, notification: crate::protocol::JsonRpcNotification) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::Disconnected);
        }
        
        let value = serde_json::to_value(&notification)
            .map_err(|e| TransportError::Json(e.to_string()))?;
        
        self.outgoing.send(value)
            .map_err(|_| TransportError::Disconnected)
    }
    
    async fn receive_message(&mut self) -> Result<Option<Value>, TransportError> {
        if !self.connected {
            return Ok(None);
        }
        
        match self.incoming.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.connected = false;
                Ok(None)
            }
        }
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        self.connected = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}
