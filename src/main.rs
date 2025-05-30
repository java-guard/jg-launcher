use crate::jvm::jvm_launcher::jvm_launch;
use args_parser::LauncherArg;

mod args_parser;
mod jar_info;
mod util;
mod base;
mod jvm;

fn main() {
    let arg = LauncherArg::get();
    // println!("{:#?}", arg);
    jvm_launch(arg);
}