//! `libra agent mcp` — repository MCP server (plan-20261019 MC-01).
//!
//! `--stdio` serves legacy MCP (`2025-11-25` / `2025-06-18`) as newline-delimited
//! JSON-RPC 2.0. stdout carries only JSON-RPC messages.

use clap::Args;
use tokio::io::{BufReader, stdin, stdout};

use crate::{
    internal::ai::mcp_server::stdio::serve,
    utils::{
        error::{CliError, CliResult},
        output::OutputConfig,
    },
};

#[derive(Args, Debug)]
pub struct McpArgs {
    /// Serve MCP on stdin/stdout. Equivalent to `--transport stdio`.
    #[arg(long)]
    pub stdio: bool,

    /// Transport name. `stdio` matches `--stdio`. `http` is not available yet.
    #[arg(long, value_name = "stdio|http")]
    pub transport: Option<String>,

    /// Internal read-tool worker. Not part of the public command surface.
    #[arg(long, hide = true)]
    pub read_worker: bool,
}

pub async fn execute_safe(args: McpArgs, _output: &OutputConfig) -> CliResult<()> {
    match selected_transport(&args)? {
        Transport::Stdio => {
            let reader = BufReader::new(stdin());
            let writer = stdout();
            serve(reader, writer).await.map_err(|error| {
                CliError::fatal(format!(
                    "libra agent mcp: fatal I/O error on the stdio transport: {error}"
                ))
            })?;
            Ok(())
        }
    }
}

enum Transport {
    Stdio,
}

fn selected_transport(args: &McpArgs) -> Result<Transport, CliError> {
    match (args.stdio, args.transport.as_deref()) {
        (false, None) => Err(usage(
            "libra agent mcp requires --stdio or --transport stdio",
        )),
        (true, None) | (false, Some("stdio")) | (true, Some("stdio")) => Ok(Transport::Stdio),
        (_, Some("http")) => Err(usage(
            "libra agent mcp: HTTP transport is not available; use --stdio",
        )),
        (stdio, Some(other)) => {
            let with_stdio = if stdio { " and --stdio" } else { "" };
            Err(usage(format!(
                "libra agent mcp: unknown --transport {other}{with_stdio}; use stdio"
            )))
        }
    }
}

fn usage(message: impl Into<String>) -> CliError {
    CliError::command_usage(message)
}
