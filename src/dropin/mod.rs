mod scope;
mod config;
mod file;

pub use file::File;
pub use config::Config;
pub use scope::{backpack_with_config, backpack};

#[cfg(test)]
mod tests {
    use crate::dropin::{backpack_with_config, Config};
    use crate::dropin::File;

    #[test]
    pub fn test_scoped() -> crate::Result<()> {
        backpack_with_config(Config::default().create_in_memory(), || {
            File::create()
        });

        Ok(())
    }
}