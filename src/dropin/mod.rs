mod scope;
mod config;
mod file;

pub use file::File;
pub use config::Config;
pub use scope::{backpack_with_config, backpack};

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Write;
    use crate::dropin::{backpack_with_config, Config};
    use crate::dropin::File;

    #[test]
    pub fn test_scoped() -> crate::Result<()> {

        backpack_with_config(
            Config::default().create_in_memory(),
            || -> io::Result<()> {
                let mut f = File::create("test.txt")?;
                writeln!(f, "yeet, this is not going to the filesystem!")?;

                Ok(())
            }
        )?;

        Ok(())
    }
}