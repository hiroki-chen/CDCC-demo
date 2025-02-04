use std::collections::HashMap;

use policy_styx_rt::PcdAppRuntime;

pub struct StyxRunner {
    policy_engine: PcdAppRuntime,
    loaded_apps: HashMap<String, PcdAppRuntime>,
}

impl StyxRunner {
    #[inline]
    pub fn new(policy_engine: PcdAppRuntime) -> Self {
        Self {
            policy_engine,
            loaded_apps: HashMap::new(),
        }
    }

    #[inline]
    pub fn load_app(&mut self, app_name: &str, app: PcdAppRuntime) {
        self.loaded_apps.insert(app_name.to_string(), app);
    }
}
