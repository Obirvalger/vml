use cmd_lib::*;

#[cmd_lib::main]
fn main() -> CmdResult {
    let opt: String = "".into();
    run_cmd!(ls $opt)
}
