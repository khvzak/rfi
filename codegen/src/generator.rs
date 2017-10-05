use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct Generator;

impl Generator {
    pub fn add_func(
            crate_name: &str,
            module_path: &[String],
            _: &str,
            fn_name: &str,
            fn_ext_name: &str,
            args_in_count: usize
    ) {
        let mut out = OpenOptions::new().append(true).create(true)
            .open(&format!("./target/rfi/{}.dat", crate_name))
            .expect("Cannot open/create rfi output file");

        write!(out,
            "{module}|{fn_name}|{fn_ext_name}|{args_in_count}\n",
            module=module_path.join("::"),
            fn_name=fn_name,
            fn_ext_name=fn_ext_name,
            args_in_count=args_in_count,
        ).unwrap();
    }
}
