use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvConfig {
    pub packages: HashMap<String, HashMap<String, Package>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub label: Option<String>,
    pub env: Option<HashMap<String, EnvAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvAction {
    pub append: Option<StringOrList>,
    pub prepend: Option<StringOrList>,
    pub set: Option<String>,
}

//
// worker device related things
//

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerDevicesOnlyConfig {
    pub devices: StandardWorkerGpuDevices,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardWorkerGpuDevices { // we use this instead of arbitrary device type map as we don't support that yet
    pub gpu: HashMap<String, StandardWorkerGpuDevice>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardWorkerGpuDevice {
    pub resources: StandardWorkerGpuDeviceResources,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardWorkerGpuDeviceResources {
    pub mem: Option<String>,
    pub opencl_ver: Option<f64>,
    pub cuda_cc: Option<f64>,
}

#[test]
fn basic() {
    let conf_text = "\
    [packages.\"houdini.py3_9\".\"19.0.720\"]\n\
    label = \"SideFX Houdini, with python version 3\"\n\
    env.PATH.prepend = \"/sw/houdinii-19.0.720/bin\"\n\
    \n\
    [packages.\"houdini.py3_11\".\"20.5.569\"]\n\
    label = \"SideFX Houdini, with python version 3\"\n\
    env.PATH.prepend = [\"/sw/houdini-20.5.569/bin\"]\n\
    ";

    let config: EnvConfig = toml::from_str(conf_text).unwrap();

    println!("{:?}", config);
}
