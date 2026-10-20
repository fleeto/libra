//! MCP server front end for `libra agent mcp`.
//!
//! Protocol constants, the tool registry, and the stdio transport live here.
//! The DeepSeek Harness bridge is a separate front end and is not modified.

pub mod protocol;
pub mod stdio;
pub mod tools;
