#[derive(Copy, Clone)]
pub enum OpenPolicy {
    OnDisk,
    /// When the file is closed, all contents are lost and
    /// and trying to open a file with the same path will
    /// fail. Creating a new file with the same path will
    /// simply open a new empty file.
    ///
    /// InMemory files can be manually added to a backpack
    InMemory,

    /// Creates a new backpack for every thread. Files are created
    /// in the thread-local backpack they were first opened in. However,
    /// Files are safe to send to other threads.
    ThreadLocalBackpack
}

#[derive(Clone)]
pub struct Config {
    pub open_policy: OpenPolicy
}

impl AsRef<Config> for Config {
    fn as_ref(&self) -> &Config {
        &self
    }
}

impl Config {
    pub fn thread_local() -> Config {
        Self {
            open_policy: OpenPolicy::ThreadLocalBackpack
        }
    }

    pub fn create_thread_local(&mut self) -> &mut Self {
        self.open_policy = OpenPolicy::ThreadLocalBackpack;
        self
    }

    pub fn create_in_memory(&mut self) -> &mut Self {
        self.open_policy = OpenPolicy::InMemory;
        self
    }

    pub fn create_on_disk(&mut self) -> &mut Self {
        self.open_policy = OpenPolicy::OnDisk;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            open_policy: OpenPolicy::OnDisk
        }
    }
}