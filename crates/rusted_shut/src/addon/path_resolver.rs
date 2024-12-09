use std::path::PathBuf;

pub trait AddonPathResolver {
    fn get_behaviour_block_base(&mut self) -> PathBuf {
        let mut base = self.get_behaviour_base();
        base.push("blocks");
        base
    }
    fn get_behaviour_item_base(&mut self) -> PathBuf {
        let mut base = self.get_behaviour_base();
        base.push("items");
        base
    }

    fn get_behaviour_block_output(&mut self, id: &str) -> PathBuf {
        let path = self.get_behaviour_block_base();

        let str = format!("{}.json", id.to_owned().replace(":", "_"));

        path.with_file_name(str)
    }
    fn get_behaviour_item_output(&mut self, id: &str) -> PathBuf {
        let path = self.get_behaviour_item_base();

        let str = format!("{}.json", id.to_owned().replace(":", "_"));

        path.with_file_name(str)
    }

    fn get_behaviour_base(&mut self) -> PathBuf;
    fn get_resource_base(&mut self) -> PathBuf;
}

pub mod default_impl {
    use crate::addon::path_resolver::AddonPathResolver;
    use std::path::PathBuf;

    pub struct BaseResolver(PathBuf);

    impl BaseResolver {
        pub fn new(output_base: PathBuf) -> Self {
            Self(output_base)
        }
    }

    impl AddonPathResolver for BaseResolver {
        fn get_behaviour_base(&mut self) -> PathBuf {
            let mut r = self.0.clone();
            r.push("BP");
            r
        }

        fn get_resource_base(&mut self) -> PathBuf {
            let mut r = self.0.clone();
            r.push("RP");
            r
        }
    }
}
