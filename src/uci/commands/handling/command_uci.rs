const AUTHOR: &str = "DerPenz";
const NAME: &str = "Chesse";

pub fn handle_setup() -> String {
    format!("id name {}\n id author {}\nuciok", NAME, AUTHOR)
}
