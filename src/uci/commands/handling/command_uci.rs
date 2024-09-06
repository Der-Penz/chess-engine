use super::command_set_option::OptionType;

const AUTHOR: &str = "DerPenz";
const NAME: &str = "Cheese";

pub fn handle_setup() -> String {
    let mut info = String::new();

    info.push_str(&format!("id name {}\n id author {}\nuciok", NAME, AUTHOR));
    OptionType::get_all_descriptions().iter().for_each(|desc| {
        info.push_str(&format!("\n{}", desc));
    });

    info
}
