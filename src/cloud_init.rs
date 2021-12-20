use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as AnyhowContext;
use anyhow::{bail, Result};
use tera::Context;

use crate::net;
use crate::template;
use crate::Error;

pub fn generate_data(context: &Context, work_dir: &Path) -> Result<PathBuf> {
    let data = work_dir.join("data.img");
    fs::create_dir_all(&work_dir)?;

    let meta_data_template: &str = "instance-id: {{ n }}\nlocal-hostname: {{ hostname }}\n";
    let meta_data = template::render(context, meta_data_template, "cloud-init meta-data")?;
    let meta_data_yaml = work_dir.join("meta-data.yaml");
    fs::write(&meta_data_yaml, &meta_data)?;

    let user_data_template: &str = "#cloud-config
disable_root: False
{% if ssh_authorized_keys -%}
ssh_authorized_keys:
{%- for key in ssh_authorized_keys %}
  - {{ key }}
{% endfor -%}
{% endif -%}
{% if users -%}
users:
{%- for user in users %}
  - name: {{ user }}
{%- if ssh_authorized_keys %}
    ssh_authorized_keys:
{%- for key in ssh_authorized_keys %}
      - {{ key }}
{% endfor -%}
{% endif -%}
{% endfor -%}
{% endif -%}
preserve_hostname: false
hostname: {{ hostname }}
";
    let user_data = template::render(context, user_data_template, "cloud-init user-data")?;
    let user_data_yaml = work_dir.join("user-data.yaml");
    fs::write(&user_data_yaml, &user_data)?;

    let mut cloud_localds = Command::new("cloud-localds");
    cloud_localds.args(&[&data, &user_data_yaml, &meta_data_yaml]);

    if let Some(address) = context.get("address").and_then(|a| a.as_str()) {
        if !net::is_cidr(address) {
            bail!(Error::BadCidr(address.to_string()));
        }
        let network_template: &str = r#"version: 2
ethernets:
  interface0:
    match:
      name: "e*"
    addresses:
      - {{ address }}
    nameservers:
      addresses:
{%- for ns in nameservers %}
      - {{ ns }}
{% endfor -%}
{%- if gateway4 %}
    gateway4: {{ gateway4 }}
{% endif -%}
{%- if gateway6 %}
    gateway6: {{ gateway6 }}
{% endif -%}
"#;
        let network = template::render(context, network_template, "cloud-init network")?;
        let network_yaml = work_dir.join("network.yaml");
        fs::write(&network_yaml, &network)?;
        cloud_localds.arg("-N").arg(&network_yaml);
    }

    cloud_localds.spawn().context("failed to run executable cloud-localds")?.wait()?;

    Ok(data)
}
