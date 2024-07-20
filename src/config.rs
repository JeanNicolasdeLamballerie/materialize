use std::path;
// use std::iter::Map;
use windows_registry::Key;

use crate::shapes::ShapeKind;

const REGISTRY_PATH_CONFIG: &str = "SOFTWARE\\Dekharen\\materialize_config\\";
const CONFIG_LIST: &str = "SOFTWARE\\Dekharen\\materialize_config\\list";
//TODO Add drag and drop for a config file ?

pub enum UpdateStatus {
    UpToDate,
    NewerAlreadyInstalled,
    OlderAlreadyInstalled,
}

#[derive(Debug, Clone)]
pub struct ConfigList(Vec<String>);
impl ConfigList {
    pub fn refresh(&mut self) {
        let reg_key = match windows_registry::CURRENT_USER.create(CONFIG_LIST) {
            Err(err) => {
                eprintln!("{}", err);
                panic!("An error has occured. See above logging.")
            }
            Ok(reg) => reg,
        };
        self.0 = reg_key.keys().unwrap().collect();
    }
    pub fn values(&self) -> Vec<String> {
        return self.0.clone();
    }
    pub fn remove(&mut self, name: &str) -> windows_registry::Result<()> {
        let reg_key = match windows_registry::CURRENT_USER.create(CONFIG_LIST) {
            Err(err) => {
                eprintln!("{}", err);
                panic!("An error has occured. See above logging.")
            }
            Ok(reg) => reg,
        };
        reg_key.remove_tree(name)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalConfiguration {
    pub cfg_list: ConfigList,
    pub configuration: Configuration,
    // pub index: u32,
    // pub len: u32,
    // pub scale: ConfigurationField<f32>,
    pub open: bool,
    pub cfg_list_open : bool,
    pub new_profile_name : String,
    pub shape_list_open : bool,
}

impl GlobalConfiguration {
    pub fn rename(&self) {}
}

impl Default for GlobalConfiguration {
   fn default() -> Self {
        let mut list = ConfigList(Vec::new());
        list.refresh();
        Self {
            cfg_list:list,
            configuration : Configuration::default(),
            open:false,
            cfg_list_open:false,
            shape_list_open:false,
            new_profile_name : String::from("New Profile"),
        }
    
}
}

#[derive(Debug, Clone)]
pub struct ConfigurationField<T> {
    pub value: T,
    pub key: String,
}
impl<T> ConfigurationField<T> {
    fn get_key(&self, name: &str) -> windows_registry::Result<Key> {
        let mut p = path::PathBuf::from(CONFIG_LIST);
        p.push(name);
        let path = match p.to_str() {
            Some(val) => val,
            _ => panic!(
                "An error occured processing the path to the configuration ! Path value : {} - {}",
                CONFIG_LIST, name
            ),
        };
        return windows_registry::CURRENT_USER.create(path); // TODO ADD NAME
    }
}

trait Access {
    fn update(&self, key_name: &str) -> windows_registry::Result<()>;
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()>;
}
// impl<T> Access for ConfigurationField<T> {
//     fn get_key() {
//         let key = windows_registry::CURRENT_USER.create(REGISTRY_PATH_CONFIG)?;
//     }
// }
impl Access for ConfigurationField<String> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        key.set_string(&self.key, &self.value)?;
        Ok(())
    }
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        self.value = key.get_string(&self.key)?;
        Ok(())
    }
}
impl Access for ConfigurationField<u32> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        key.set_u32(&self.key, self.value)?;
        Ok(())
    }
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        self.value = key.get_u32(&self.key)?;
        Ok(())
    }
}
impl Access for ConfigurationField<f32> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        key.set_string(&self.key, &self.value.to_string())?;
        Ok(())
    }
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        self.value = key.get_string(&self.key)?.parse::<f32>().unwrap(); //TODO Needs to throw
                                                                         //error one way or another
        Ok(())
    }
}

impl Access for ConfigurationField<(f32, f32)> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        let k1 = self.key.clone() + "_x";
        let k2 = self.key.clone() + "_y";
        key.set_u32(&k1, self.value.0 as u32)?;
        key.set_u32(&k2, self.value.1 as u32)?;
        Ok(())
    }
    /// Forcibly executes and converts f32 into u32 & vice versa.
    /// If you need a specific, actual f32 number, use a string & conversion instead.
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        let k1 = self.key.clone() + "_x";
        let k2 = self.key.clone() + "_y";
        let v1 = key.get_u32(k1)?;
        let v2 = key.get_u32(k2)?;
        self.value = (v1 as f32, v2 as f32);
        Ok(())
    }
}
impl Access for ConfigurationField<bool> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        match self.value {
            true => key.set_u32(&self.key, 1)?,
            false => key.set_u32(&self.key, 0)?,
        }
        Ok(())
    }
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        let stored = key.get_u32(&self.key)?;

        match stored {
            1 => self.value = true,
            0 => self.value = false,
            invalid_value => panic!(
                "Registry boolean is corrupted ! See the actual u32 value : {}",
                invalid_value
            ),
        }
        Ok(())
    }
}

impl Access for ConfigurationField<usize> {
    fn update(&self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        key.set_u32(&self.key, self.value as u32)?;
        Ok(())
    }
    fn retrieve_from_registry(&mut self, key_name: &str) -> windows_registry::Result<()> {
        let key = self.get_key(key_name)?;
        self.value = key.get_u32(&self.key)? as usize;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub version: ConfigurationField<String>,
    pub screen_size: ConfigurationField<(f32, f32)>,
    pub size_arr: ConfigurationField<usize>,
    pub viewed_frequencies: ConfigurationField<u32>,
    pub polled_frequencies: ConfigurationField<u32>,
    pub double_precision: ConfigurationField<bool>,
    pub number_of_items: ConfigurationField<u32>,
    pub scale: ConfigurationField<f32>,
    // pub terminal_display: bool,
    //

    // |-----------------------|
    pub key: String, //|
    // |string --- Unique key ?|
    // |-----------------------|
    // |Needs to be enforced at|
    // |a higher level.        |
    // |-----------------------|
    pub kind: ShapeKind,
}

const VERSION: &str = "0.0.1";
impl Default for Configuration {
    fn default() -> Self {
        Self {
            kind: ShapeKind::RoundedRectangular,
            key: String::from("default"),
            scale: ConfigurationField {
                value: 100.0,
                key: "scale".to_string(),
            },
            version: ConfigurationField {
                key: "version".to_string(),
                value: VERSION.into(),
            },
            number_of_items: ConfigurationField {
                value: 64,
                key: "number_of_items".to_string(),
            },

            double_precision: ConfigurationField {
                value: true,
                key: "double_precision".to_string(),
            },
            viewed_frequencies: ConfigurationField {
                key: "viewed_frequencies".to_string(),
                value: 2000,
            },
            screen_size: ConfigurationField {
                key: "screen_size".to_string(),
                value: (1920.0, 1080.0),
            },
            size_arr: ConfigurationField {
                key: "size_arr".to_string(),
                value: 8192,
            },
            polled_frequencies: ConfigurationField {
                key: "polled_frequencies".to_string(),
                value: 4000,
            },
        }
    }
}
impl Configuration {
    pub fn status(&self) -> windows_registry::Result<UpdateStatus> {
        let mut default = Configuration::default();
        default.version.retrieve_from_registry(&self.key)?;

        let stat = match default.version.value {
            stored_version if stored_version < self.version.value => {
                UpdateStatus::OlderAlreadyInstalled
            }

            stored_version if stored_version == self.version.value => UpdateStatus::UpToDate,

            _ => UpdateStatus::NewerAlreadyInstalled,
        };
        Ok(stat)
    }
    // fn get_key(&self) -> windows_registry::Result<Key> {
    //     return windows_registry::CURRENT_USER.create(REGISTRY_PATH_CONFIG);
    // }
    //
//TODO UPDATE THIS FUNCTION TO LOOKUP ACTUAL PATH
    pub fn exists(&self) -> bool {
        match windows_registry::CURRENT_USER.open(REGISTRY_PATH_CONFIG) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
    pub fn new(
        size_arr: usize,
        polled_frequencies: u32,
        screen_size: (f32, f32),
        double_precision: bool,
        number_of_items: u32,
        scale: f32,
        key: String,
        kind: ShapeKind,
    ) -> Self {
        let viewed_frequencies = if double_precision {
            polled_frequencies / 2
        } else {
            polled_frequencies
        };

        return Self {
            key,
            kind,
            scale: ConfigurationField {
                value: scale,
                key: "scale".to_string(),
            },
            version: ConfigurationField {
                key: "version".to_string(),
                value: VERSION.into(),
            },
            number_of_items: ConfigurationField {
                value: number_of_items,
                key: "number_of_items".to_string(),
            },

            double_precision: ConfigurationField {
                value: double_precision,
                key: "double_precision".to_string(),
            },
            viewed_frequencies: ConfigurationField {
                key: "viewed_frequencies".to_string(),
                value: viewed_frequencies,
            },
            screen_size: ConfigurationField {
                key: "screen_size".to_string(),
                value: screen_size,
            },
            size_arr: ConfigurationField {
                key: "size_arr".to_string(),
                value: size_arr,
            },
            polled_frequencies: ConfigurationField {
                key: "polled_frequencies".to_string(),
                value: polled_frequencies,
            },
        };
    }
    // fn get_key(&self) -> windows_registry::Result<Key> {
    //     return windows_registry::CURRENT_USER.create(REGISTRY_PATH_CONFIG);
    // }
    pub fn switch_precision(&mut self) {
        self.double_precision.value = !self.double_precision.value;
    }
    pub fn update_to_registry(&mut self) -> windows_registry::Result<()> {
        let kind =self.kind.to_str();
        let field = ConfigurationField {
            key:String::from("kind"),
            value:String::from(kind),
        };
        field.update(&self.key)?;
        self.version.update(&self.key)?;
        self.polled_frequencies.update(&self.key)?;
        self.viewed_frequencies.update(&self.key)?;
        self.screen_size.update(&self.key)?;
        self.double_precision.update(&self.key)?;
        self.size_arr.update(&self.key)?;
        Ok(())
    }
    pub fn retrieve_from_registry(&mut self) -> windows_registry::Result<()> {
        let kind = self.kind.to_str();
        let mut field = ConfigurationField {
            key:String::from("kind"),
            value:String::from(kind),
        };
        field.retrieve_from_registry(&self.key)?;
        self.kind = ShapeKind::from_str(&field.value);
        self.version.retrieve_from_registry(&self.key)?;
        self.polled_frequencies.retrieve_from_registry(&self.key)?;
        self.viewed_frequencies.retrieve_from_registry(&self.key)?;
        self.screen_size.retrieve_from_registry(&self.key)?;
        self.double_precision.retrieve_from_registry(&self.key)?;
        self.size_arr.retrieve_from_registry(&self.key)?;
        Ok(())
    }
    pub fn set(&self) -> windows_registry::Result<()> {
        // let REGISTRY_PATH_CONFIG = "SOFTWARE\\Dekharen\\materialize_config\\";
        let key = windows_registry::CURRENT_USER.create(REGISTRY_PATH_CONFIG)?;
        key.set_u32("polled_frequencies", self.polled_frequencies.value)?;
        key.set_string("hello", "world")?;
        Ok(())
    }
}
