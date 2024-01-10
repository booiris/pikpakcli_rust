use pikpakcli::run_cmd;

fn main() -> Result<(), anyhow::Error> {
    if let Err(err) = run_cmd() {
        log::error!("Error: {:?}", err);
        return Err(err);
    }
    Ok(())
}
