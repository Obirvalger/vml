use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ModifyCommandQemu {
    #[serde(default)]
    pub append: Vec<String>,
    #[serde(default)]
    pub prepend: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ModifyCommand {
    #[serde(default)]
    pub qemu: ModifyCommandQemu,
}
