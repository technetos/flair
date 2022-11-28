pub struct Arg {
    pub name: String,
    pub value: String,
}

fn args_impl(input: impl Iterator<Item = String>) -> Vec<Arg> {
    input
        .skip(1)
        .map(|arg| {
            let pos = arg.chars().position(|v| v == '=');
            match arg {
                s if s.starts_with("-") && pos.is_some() => {
                    let pos = pos.unwrap();
                    let name = String::from(&s[1..pos]);
                    let value = String::from(&s[pos + 1..]);

                    Arg { name, value }
                }
                _ => panic!("Invalid commandline arguments"),
            }
        })
        .collect()
}

pub fn args() -> Vec<Arg> {
    args_impl(std::env::args())
}
