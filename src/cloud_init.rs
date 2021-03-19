use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tera::Context;

use crate::template;
use crate::{Error, Result};

pub fn generate_data(context: &Context, work_dir: &PathBuf) -> Result<PathBuf> {
    let data = work_dir.join("data.img");
    fs::create_dir_all(&work_dir)?;

    let meta_data_template: &str = "instance-id: {{ n }}\nlocal-hostname: {{ n }}\n";
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
hostname: {{ n }}
";
    let user_data = template::render(context, user_data_template, "cloud-init user-data")?;
    let user_data_yaml = work_dir.join("user-data.yaml");
    fs::write(&user_data_yaml, &user_data)?;

    let mut cloud_localds = Command::new("cloud-localds");
    cloud_localds.args(&[&data, &user_data_yaml, &meta_data_yaml]);
    cloud_localds
        .spawn()
        .map_err(|e| Error::executable("cloud-localds", &e.to_string()))?
        .wait()?;

    Ok(data)
}
