use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = your_module::stub_info()?;
    stub.generate()?;
    Ok(())
}
