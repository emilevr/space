use std::{any::Any, io::Write};

pub trait CliCommand {
    fn prepare(&mut self) -> anyhow::Result<()>;
    fn run<W: Write>(&mut self, writer: &mut W) -> anyhow::Result<()>;
    fn as_any(&self) -> &dyn Any;
}
