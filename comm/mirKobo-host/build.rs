use std::env;

fn main() {
    let dir_let = env::current_dir().unwrap().join("vcpkg");
    let dir = dir_let.to_str().unwrap();
    println!("env::current_dir() {:?}", dir);
    env::set_var("VCPKG_ROOT", dir);
    // std::process::exit(1);
}