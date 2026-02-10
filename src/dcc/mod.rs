//! DCC (Direct Client-to-Client) file transfer subsystem.
//!
//! Supports receiving files via the DCC SEND protocol with security protections
//! including path traversal prevention, private IP rejection, and file size limits.

pub mod manager;
pub mod parser;
pub mod search_results;
pub mod security;
pub mod transfer;
