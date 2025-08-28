use std::time::Instant;

use crate::Result;

#[derive(Debug)]
pub struct Elapsed(Instant);

impl Default for Elapsed {
    fn default() -> Self {
        Elapsed(Instant::now())
    }
}

impl Elapsed {
    pub fn finish(&self, msg: &str) -> Result<()> {
        let elapsed = Instant::now().duration_since(self.0);
        log::info!("{} elapsed:{:?}", msg, elapsed);
        Ok(())
    }
}
