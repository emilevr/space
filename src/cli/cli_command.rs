use std::io::Write;

pub trait CliCommand {
    fn prepare(&mut self) -> anyhow::Result<&mut Self>;
    fn run<W: Write>(&mut self, writer: &mut W) -> anyhow::Result<()>;
}
